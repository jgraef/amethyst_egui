#!/bin/sh

SHADERS="shaders/"
COMPILED="$SHADERS/compiled"
SRC="$SHADERS/src"

glslc -g -c -O -o "$COMPILED/egui.vert.spv" "$SRC/egui.vert"
glslc -g -c -O -o "$COMPILED/egui.frag.spv" "$SRC/egui.frag"

