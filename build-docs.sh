#!/bin/bash
# This script is used to build the documentation for the project.

# Build the documentation
cargo doc

# Copy the documentation to the docs directory
cp -r target/doc docs

# Add a redirect to the index.html file
echo "<meta http-equiv=refresh content=0;url=sage_lisp/index.html>" > docs/index.html