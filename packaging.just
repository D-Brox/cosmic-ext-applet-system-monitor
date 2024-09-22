# Compiles and packages deb with release profile
build-deb:
    command -v cargo-deb || cargo install cargo-deb
    cargo deb

install-deb:
    apt install --reinstall ./target/debian/*.deb

# Compiles and packages rpm with release profile
build-rpm: build-release
    command -v cargo-generate-rpm || cargo install cargo-generate-rpm
    strip -s {{bin-src}}
    cargo generate-rpm

install-rpm:
    dnf localinstall ./target/generate-rpm/*.rpm

# Compiles and packages PKGBUILD with release profile
build-aur:
    command -v cargo-aur || cargo install cargo-aur
    cargo aur

install-aur:
    makepkg -siD ./target/cargo-aur

# Flatpak recipes

fp-prefix := '/app'
fp-base-dir := absolute_path(clean(rootdir / fp-prefix))
fp-bin-dst := fp-base-dir / 'bin' / name
fp-desktop-dst := fp-base-dir / 'share' / 'applications' / desktop
fp-metainfo-dst := fp-base-dir / 'share' / 'metainfo' / metainfo
fp-icons-dst := fp-base-dir / 'share' / 'icons' / 'hicolor'

update-flatpak:
    command -v cargo-flatpak || cargo install --git https://github.com/D-Brox/cargo-flatpak.git
    cargo flatpak

# # Replace "<repo>" with the flatpak repo
# publish-flatpak: update-flatpak
#     flatpak-builder --repo=<repo>  build {{app_id}}.json

install-flatpak: update-flatpak
    flatpak-builder --install  build {{APPID}}.json

[private]
flatpak-builder:
    cargo --offline fetch --manifest-path Cargo.toml --verbose
    cargo --offline build --release --verbose
    install -Dm0755 {{bin-src}} {{fp-bin-dst}}
    install -Dm0644 {{desktop-src}} {{fp-desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{fp-metainfo-dst}}
    install -Dm0644 "{{icons-src}}/scalable/apps/{{APPID}}.svg" "{{fp-icons-dst}}/scalable/apps/{{APPID}}.svg"