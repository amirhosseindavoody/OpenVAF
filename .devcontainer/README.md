# OpenVAF Docker

This project provides a Docker image for OpenVAF, which includes a pre-configured environment with essential tools and libraries for development.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Building the Docker Image](#building-the-docker-image)
- [Running the Docker Container](#running-the-docker-container)
- [Usage](#usage)
- [License](#license)

## Prerequisites
- Docker must be installed on your machine. You can download it from [Docker's official website](https://www.docker.com/get-started).

## Building the Docker Image
To build the Docker image, navigate to the project directory and run the following command:

```bash
docker build -t openvaf-docker .
```

This command will create a Docker image named `openvaf-docker` based on the instructions specified in the `Dockerfile`.

## Running the Docker Container
Once the image is built, you can run a container using the following command:

```bash
docker run -it openvaf-docker
```

This will start an interactive terminal session inside the container.

## Usage
After starting the container, you will have access to the following tools and libraries:
- Zsh as the default shell
- LLVM 16 for C/C++ development
- Rust (version 1.65.0)

You can start using these tools directly in the terminal.

## License
This project is licensed under the MIT License. See the LICENSE file for more details.