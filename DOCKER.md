# Running electrs-junkcoin with Docker

This document provides instructions for running electrs-junkcoin using Docker.

## Prerequisites

### Installing Docker

1. Update package index and install required packages:
```bash
sudo apt-get update
sudo apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release
```

2. Add Docker's official GPG key:
```bash
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
```

3. Set up the stable repository:
```bash
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
```

4. Install Docker Engine:
```bash
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io
```

5. Add your user to the docker group:
```bash
sudo usermod -aG docker junk
newgrp docker
```

6. Verify Docker installation:
```bash
docker --version
docker run hello-world
```

Note: After adding your user to the docker group, you may need to log out and log back in for the changes to take effect.

## Building the Docker Image

1. Build the Docker image:
```bash
cd electrs-junkcoin
docker build -t electrs-junkcoin .
```

## Running the Container

The following command will run the electrs-junkcoin container with host networking to access the Junkcoin daemon:

```bash
docker run -d \
  --name electrs-junkcoin \
  --network host \
  -v /home/junk/.junkcoin:/home/explore/.junkcoin \
  -v /home/junk/.electrs:/electrs \
  -e HTTP_ADDR=0.0.0.0:50010 \
  -e DB_DIR=/electrs \
  -e DAEMON_DIR=/home/explore/.junkcoin \
  -e DAEMON_RPC_ADDR=127.0.0.1:9771 \
  -e NETWORK=mainnet \
  -e MONITORING_ADDR=0.0.0.0:4224 \
  electrs-junkcoin
```

### Configuration Breakdown

#### Network Mode
- `--network host`: Uses host networking to access the Junkcoin daemon

#### Volume Mounts
- `-v /home/junk/.junkcoin:/home/explore/.junkcoin`: Maps the Junkcoin data directory
- `-v /home/junk/.electrs:/electrs`: Maps the Electrs database directory

#### Environment Variables
- `HTTP_ADDR=0.0.0.0:50010`: HTTP server address and port
- `DB_DIR=/electrs`: Directory for the electrs database
- `DAEMON_DIR=/home/explore/.junkcoin`: Junkcoin data directory inside container
- `DAEMON_RPC_ADDR=127.0.0.1:9771`: Junkcoin RPC address and port
- `NETWORK=mainnet`: Network type
- `MONITORING_ADDR=0.0.0.0:4224`: Prometheus monitoring address

## Monitoring

The service exposes Prometheus metrics on port 4224.

## Logs and Troubleshooting

View container logs:
```bash
docker logs electrs-junkcoin
```

Common issues:
- Ensure Junkcoin daemon (junkcoind) is running and accessible
- Verify RPC credentials in junkcoin.conf
- Check if daemon is fully synced
- Ensure host networking is properly configured
- Verify volume mount paths exist and have correct permissions

## Managing the Container

Stop the container:
```bash
docker stop electrs-junkcoin
```

Start the container:
```bash
docker start electrs-junkcoin
```

Remove the container:
```bash
docker rm electrs-junkcoin
```

View container status:
```bash
docker ps -a | grep electrs-junkcoin
```

## Updating

To update to a new version:

```bash
docker stop electrs-junkcoin
docker rm electrs-junkcoin
docker pull electrs-junkcoin:latest
# Then run the container again with the configuration above
```

## Resource Requirements

- CPU: 2+ cores recommended
- RAM: 2GB minimum
- Disk: Depends on blockchain size and index
- Network: Stable internet connection

## Security Considerations

- Always use strong RPC passwords
- Consider running behind a reverse proxy
- Restrict network access to RPC and monitoring ports
- Use Docker networks to isolate containers when possible
- Regularly update Docker and containers for security patches
