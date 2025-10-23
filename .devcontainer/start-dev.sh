#!/bin/sh




if [ -f /workspace/frontend/package.json ]; then 
    cd /workspace/frontend \
    && npm install \
    && npx vite --host
else
    npm create vite@latest frontend --
fi

