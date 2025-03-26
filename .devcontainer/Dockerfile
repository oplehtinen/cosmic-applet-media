# from https://jasoncg.dev/blog/2023/developing-gui-app-bevy-in-vscode-devcontainer-wayland/
FROM mcr.microsoft.com/devcontainers/rust:dev-1-bookworm 

# Install the necessary system libraries
RUN apt update && apt install -y \   
    build-essential \
    pkg-config \    
    libasound2-dev \
    libudev-dev \   
    mesa-utils \ 
    vulkan-tools \
    libwayland-dev \
    libxkbcommon-dev \   
    libvulkan1 \    
    libvulkan-dev \ 
    libegl1-mesa-dev \   
    libgles2-mesa-dev \  
    libx11-dev \    
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxrandr-dev \
    libxcb1-dev \
    libxcb-icccm4-dev \
    libxcb-image0-dev \
    libxcb-keysyms1-dev \
    libxcb-randr0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxcb-xkb-dev \
    libegl1-mesa \
    libgl1-mesa-glx \
    libgl1-mesa-dri \
    libglu1-mesa-dev \
    libglu1-mesa \
    libgles2-mesa \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*