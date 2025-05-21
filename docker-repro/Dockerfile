FROM ghcr.io/prefix-dev/pixi:0.46.0-noble AS pixi-installer
USER root
COPY pixi-install-to-prefix/pixi.* /etc/pixi-install-to-prefix/
RUN pixi install --manifest-path /etc/pixi-install-to-prefix/pixi.toml --frozen

FROM ghcr.io/prefix-dev/pixi:0.44.0-noble
USER root
COPY --from=pixi-installer /etc/pixi-install-to-prefix/ /etc/pixi-install-to-prefix/
COPY test-env/pixi.* ./build/

# After running docker build -t dns-repro:latest .
# run: docker run -it dns-repro:latest /bin/bash
# and then the following command to reproduce the issue:
# pixi run --manifest-path /etc/pixi-install-to-prefix --no-lockfile-update pixi-install-to-prefix --lockfile ./build/pixi.lock /opt/conda/envs/test-env
