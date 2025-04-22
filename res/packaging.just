rootdir := ''
prefix := '/usr'
base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-path := name
bin-dst := base-dir / 'bin' / bin-path

desktop := APPID + '.desktop'
desktop-src := 'res' / desktop
desktop-path := 'applications' / desktop
desktop-dst := base-dir / 'share' / desktop-path

metainfo := APPID + '.metainfo.xml'
metainfo-src := 'res' / metainfo
metainfo-path := 'metainfo' / metainfo
metainfo-dst := base-dir / 'share' / metainfo-path

icon := APPID + '.svg'
icon-src := 'res' / icon
icon-path := 'icons' / 'hicolor' / 'scalable' / 'apps' / icon
icon-dst := base-dir / 'share' / icon-path

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile with wgpu disabled
build-no-wgpu *args: (build-debug '--release --no-default-features' args)

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Installs files globally
[no-cd]
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}
    install -Dm0644 {{icon-src}} {{icon-dst}}

# Uninstalls globally installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{metainfo-dst}}
    rm {{icon-dst}}


# Installs files locally
[no-cd]
install-local:
    install -Dm0755 {{bin-src}} ~/.local/bin/{{bin-path}}
    install -Dm0644 {{desktop-src}} ${XDG_DATA_HOME:-~/.local/share}/{{desktop-path}}
    install -Dm0644 {{metainfo-src}} ${XDG_DATA_HOME:-~/.local/share}/{{metainfo-path}}
    install -Dm0644 {{icon-src}} ${XDG_DATA_HOME:-~/.local/share}/{{icon-path}}

# Uninstalls locally installed files
uninstall-local:
    rm ~/.local/bin/{{bin-path}}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{desktop-path}}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{metainfo-path}}
    rm ${XDG_DATA_HOME:-~/.local/share}/{{icon-path}}

# Compiles and packages deb with release profile
build-deb:
    command -v cargo-deb || cargo install cargo-deb
    cargo deb

[no-cd]
install-deb:
    apt install --reinstall ./target/debian/*.deb

# Compiles and packages rpm with release profile
[no-cd]
build-rpm: build-release
    command -v cargo-generate-rpm || cargo install cargo-generate-rpm
    strip -s {{bin-src}}
    cargo generate-rpm

[no-cd]
install-rpm:
    dnf install ./target/generate-rpm/*.rpm

install-aur aur_helper:
    {{aur_helper}} -S {{name}}-git

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Vendor dependencies locally
[no-cd]
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
[no-cd]
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar