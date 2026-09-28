# Build with the pinned official gaia2-cli directory as the context. Put the
# Fenix adapter at fenix/ and the frozen native executable/libraries at bundle/.
ARG GAIA2_BASE=fenix-eval-gaia2-base:87ebd38
FROM debian:bookworm-slim AS wrapper
RUN apt-get update && apt-get install -y --no-install-recommends gcc libc6-dev && rm -rf /var/lib/apt/lists/*
COPY shared/gaia2-exec.c /tmp/gaia2-exec.c
RUN gcc -O2 -o /tmp/gaia2-exec /tmp/gaia2-exec.c
FROM ${GAIA2_BASE}
RUN groupadd benchmark-transport && useradd -r -s /usr/sbin/nologin -g benchmark-transport gaia2 && useradd -m -s /bin/bash -u 1000 -G benchmark-transport agent && mkdir -p /var/gaia2/state /home/agent/bin && chown gaia2:benchmark-transport /var/gaia2 && chmod 700 /var/gaia2
COPY --from=wrapper /tmp/gaia2-exec /usr/local/bin/gaia2-exec
RUN chown gaia2:benchmark-transport /usr/local/bin/gaia2-exec && chmod 4755 /usr/local/bin/gaia2-exec && for cmd in calendar contacts emails messages chats rent-a-flat city cabs shopping cloud-drive; do ln -s /usr/local/bin/gaia2-exec /home/agent/bin/$cmd; done
COPY shared/gaia2_adapter_base.py shared/render_agent_prompt.py shared/AGENTS_TEMPLATE.md /opt/
COPY fenix/adapter.py fenix/entrypoint.py fenix/judge_daemon.py /opt/
RUN printf '#!/bin/sh\nexec python3 /opt/entrypoint.py\n' > /opt/gaia2-init-entrypoint.sh && chmod 755 /opt/gaia2-init-entrypoint.sh
COPY bundle/ /opt/fenix/
WORKDIR /home/agent
CMD ["python3", "/opt/entrypoint.py"]
