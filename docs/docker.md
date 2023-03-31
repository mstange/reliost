# Docker

In this document, you can find some basic commands like creating and handling a docker container for this project.

## Basic commands

### Build the docker image

All the instructions for building a docker image is located in the [Dockerfile](../Dockerfile).

To be able to build the image, run:

```bash
docker build -t reliost:dev .
```

Additionally, `--rm` can be passed to remove the intermediate containers after the build. Also, `--pull` can be passed to update the base images that are being used inside the Dockerfile.

The new created image will have the `reliost:dev` tag.

### Run the docker image in a container

The built image can be run with the following command:

```bash
docker run -t -i --rm -p 8080:8080 --name reliost reliost:dev
```

Here is a quick explanation of the command line parameters:

- `-t` allocates a pseudo-tty, while `-i` makes the container interactive. These
  two parameters makes it possible to interact with the container. Especially
  Ctrl-C can gracefully stop the container.
- `--rm` removes the container when it stops.
- `-p 8080:8080` makes the container listen on port 8080, and binds it to the
  inner port 8080 which is the default port configured while building the
  container.
- `--name` gives a name to this container so that it's easier to reference in commands.
