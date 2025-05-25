use sysinfo::Components;

/// Main struct for CPU temperature monitoring
pub struct CpuThermalMonitor {
    components: Components,
}

// CPU architecture types for specific sensor patterns
#[derive(Debug, Clone, Copy)]
enum CpuArchitecture {
    Intel,
    Amd,
    Arm,
    Unknown,
}

// Moved SensorData struct definition here to be accessible by helper methods
struct SensorData {
    label: String,
    temp: f32,
    priority: u8, // Higher number = higher priority
}

impl CpuThermalMonitor {
    /// Create a new CPU temperature monitor
    pub fn new() -> Self {
        Self {
            components: Components::new_with_refreshed_list(),
        }
    }

    /// Refresh temperature sensors
    pub fn refresh(&mut self) {
        self.components = Components::new_with_refreshed_list();
    }

    /// Get the current CPU temperature in Celsius
    /// `vendor_id`: From `sysinfo::Cpu::vendor_id()`
    /// `arch_family`: From `sysinfo::System::cpu_arch()`
    pub fn temperature(&self, vendor_id: &str, arch_family: &str) -> f32 {
        self.select_cpu_temperature(vendor_id, arch_family)
    }

    // Determine CpuArchitecture enum based on sysinfo provided strings
    fn determine_architecture_for_sensor_selection(
        vendor_id: &str,
        arch_family: &str,
    ) -> CpuArchitecture {
        let lower_vendor_id = vendor_id.to_lowercase();
        if lower_vendor_id.contains("amd") || lower_vendor_id.contains("authenticamd") {
            return CpuArchitecture::Amd;
        }
        if lower_vendor_id.contains("intel") || lower_vendor_id.contains("genuineintel") {
            return CpuArchitecture::Intel;
        }
        // Check arch_family for ARM after vendor specific checks
        let lower_arch_family = arch_family.to_lowercase();
        if lower_arch_family.contains("arm") || lower_arch_family.contains("aarch64") {
            return CpuArchitecture::Arm;
        }
        CpuArchitecture::Unknown
    }

    // Helper function to discover and prioritize sensors
    fn discover_and_prioritize_sensors(
        &self,
        primary_patterns: &[&str],
        secondary_patterns: &[&str],
        min_temp: f32,
        max_temp: f32,
    ) -> Vec<SensorData> {
        let mut discovered_sensors: Vec<SensorData> = Vec::new();
        for component in &self.components {
            let label = component.label().to_lowercase();
            if let Some(temp) = component.temperature() {
                if temp < min_temp || temp > max_temp {
                    continue;
                }

                let mut matched = false;
                for &pattern in primary_patterns {
                    if label == pattern {
                        discovered_sensors.push(SensorData {
                            label: component.label().to_string(),
                            temp,
                            priority: 3,
                        });
                        matched = true;
                        break;
                    }
                }

                if matched {
                    continue;
                }

                for &pattern in secondary_patterns {
                    if label.contains(pattern) {
                        discovered_sensors.push(SensorData {
                            label: component.label().to_string(),
                            temp,
                            priority: 2,
                        });
                        matched = true;
                        break;
                    }
                }

                if !matched {
                    discovered_sensors.push(SensorData {
                        label: component.label().to_string(),
                        temp,
                        priority: 1,
                    });
                }
            }
        }
        discovered_sensors
    }

    // Helper function to calculate final temperature from a list of prioritized sensors
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )] // Inherit allowances if casts are here
    fn calculate_final_temperature_from_prioritized_sensors(
        high_priority_sensors: &[SensorData],
    ) -> Option<f32> {
        match high_priority_sensors.len() {
            0 => None, // No sensors, no temperature
            1 => Some(high_priority_sensors[0].temp),
            len if len <= 3 => {
                // Covers 2 or 3 sensors
                let avg = high_priority_sensors.iter().map(|s| s.temp).sum::<f32>() / len as f32; // Use len directly
                Some(avg)
            }
            _ => {
                // Covers > 3 sensors
                let mut temps: Vec<f32> = high_priority_sensors.iter().map(|s| s.temp).collect();
                temps.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let idx = ((temps.len() as f32) * 0.95) as usize;
                let idx = idx.min(temps.len() - 1);
                Some(temps[idx])
            }
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    fn select_cpu_temperature(&self, vendor_id: &str, arch_family: &str) -> f32 {
        // SensorData struct is now defined outside this function

        // Determine CPU architecture using passed-in sysinfo data
        let cpu_arch = Self::determine_architecture_for_sensor_selection(vendor_id, arch_family);

        // Architecture-specific patterns
        let (primary_patterns, secondary_patterns, min_temp, max_temp) = match cpu_arch {
            CpuArchitecture::Amd => {
                // AMD-specific sensors
                let primary = &["tctl", "cpu temperature"][..];
                let secondary = &["cpu", "tccd", "core", "package"][..];
                (primary, secondary, 10.0, 110.0)
            }
            CpuArchitecture::Intel => {
                // Intel-specific sensors
                let primary = &["package id 0", "cpu temperature", "coretemp"][..];
                let secondary = &["core", "cpu", "package"][..];
                (primary, secondary, 10.0, 105.0)
            }
            CpuArchitecture::Arm => {
                // ARM-specific sensors
                let primary = &["cpu-thermal", "soc"][..];
                let secondary = &["thermal", "cpu", "core"][..];
                (primary, secondary, 10.0, 90.0) // ARM CPUs typically run cooler
            }
            CpuArchitecture::Unknown => {
                // Generic fallback patterns
                let primary = &["cpu temperature"][..];
                let secondary = &["cpu", "core", "package", "thermal"][..];
                (primary, secondary, 10.0, 110.0)
            }
        };

        let mut discovered_sensors = self.discover_and_prioritize_sensors(
            primary_patterns,
            secondary_patterns,
            min_temp,
            max_temp,
        );

        // Calculate final temperature based on discovered sensors
        if !discovered_sensors.is_empty() {
            // Sort by priority (descending)
            discovered_sensors.sort_by(|a, b| b.priority.cmp(&a.priority));

            // Get the highest priority level available
            let highest_priority = discovered_sensors[0].priority;

            // Filter to only the highest priority sensors
            let high_priority_sensors_vec: Vec<SensorData> = discovered_sensors
                .into_iter() // consume discovered_sensors to move data
                .filter(|s| s.priority == highest_priority)
                .collect();

            // Detailed temperature selection based on architecture
            match cpu_arch {
                CpuArchitecture::Amd => {
                    // For AMD, prefer Tctl sensor if available at highest priority
                    let tctl_sensor = high_priority_sensors_vec
                        .iter()
                        .find(|s| s.label.to_lowercase() == "tctl");

                    if let Some(sensor) = tctl_sensor {
                        return sensor.temp;
                    }

                    // If multiple CCDs, take the highest temperature (thermal throttling based on hottest core)
                    let tccd_sensors: Vec<_> = high_priority_sensors_vec
                        .iter()
                        .filter(|s| s.label.to_lowercase().contains("tccd"))
                        .collect();

                    if !tccd_sensors.is_empty() {
                        let max_temp = tccd_sensors.iter().map(|s| s.temp).fold(0.0_f32, |a, b| {
                            if a > b {
                                a
                            } else {
                                b
                            }
                        });
                        return max_temp;
                    }
                }
                CpuArchitecture::Intel => {
                    // For Intel, the package temperature usually represents the overall CPU temperature
                    // Check for package temperature among high priority sensors
                    let package_sensors: Vec<_> = high_priority_sensors_vec
                        .iter()
                        .filter(|s| s.label.to_lowercase().contains("package"))
                        .collect();

                    if !package_sensors.is_empty() {
                        let package_temp = package_sensors[0].temp;
                        return package_temp;
                    }

                    // If we have core temperatures, use the highest (throttling happens based on hottest core)
                    let core_sensors: Vec<_> = high_priority_sensors_vec
                        .iter()
                        .filter(|s| s.label.to_lowercase().contains("core"))
                        .collect();

                    if core_sensors.len() > 1 {
                        let max_temp = core_sensors.iter().map(|s| s.temp).fold(0.0_f32, |a, b| {
                            if a > b {
                                a
                            } else {
                                b
                            }
                        });
                        return max_temp;
                    } else if let Some(core_sensor) = core_sensors.first() {
                        // Handle single core sensor case
                        return core_sensor.temp;
                    }
                }
                CpuArchitecture::Arm => {
                    // For ARM, the main thermal sensor is usually the most reliable
                    // If we have a cpu-thermal or soc sensor, prefer that
                    let thermal_sensor = high_priority_sensors_vec.iter().find(|s| {
                        s.label.to_lowercase().contains("cpu-thermal")
                            || s.label.to_lowercase().contains("soc")
                    });

                    if let Some(sensor) = thermal_sensor {
                        return sensor.temp;
                    }
                }
                CpuArchitecture::Unknown => { /* Fall through to default handling */ }
            }

            // Default handling for all architectures if no special cases were triggered
            if let Some(temp) = Self::calculate_final_temperature_from_prioritized_sensors(
                &high_priority_sensors_vec,
            ) {
                return temp;
            }
        }

        // If we got here, we couldn't find any suitable sensors or len was 0
        0.0
    }
}
