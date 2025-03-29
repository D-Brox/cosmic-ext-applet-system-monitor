#!/usr/bin/env bash

mkdir -p ~/.local/bin/ 
mkdir -p ${XDG_DATA_HOME:-~/.local/share}/{applications,metainfo,icons/hicolor/scalable/apps}/
cp ./cosmic-ext-applet-privacy-indicator ~/.local/bin/
cp ./*.desktop ${XDG_DATA_HOME:-~/.local/share}/applications/
cp ./*.metainfo.xml ${XDG_DATA_HOME:-~/.local/share}/metainfo/
cp ./*.svg ${XDG_DATA_HOME:-~/.local/share}/icons/hicolor/scalable/apps/