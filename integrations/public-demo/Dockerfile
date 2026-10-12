FROM python:3.12-slim-bookworm@sha256:7753c33391fc9f01d1984375bf375eb6686d52ba10db6043a86634a5ccf90dcf
LABEL org.opencontainers.image.source="https://github.com/AlitheaBio/froglet" \
      dev.froglet.runtime.release="v0.4.7-beta.1" \
      dev.froglet.runtime.source="d70cb50017c7c526d1f7ad4faf278da07e8248a4" \
      dev.froglet.runtime.sha256="5afdbaf98f7aa2eb10cff3c29dbeca25d099e421a46a4cff40439aeaf3a25b77"
WORKDIR /app
COPY froglet-node provider_proxy.py entrypoint.py catalog-source.json ./
RUN python3 -c "import hashlib,pathlib; p=pathlib.Path('/app/froglet-node'); assert hashlib.sha256(p.read_bytes()).hexdigest()=='5afdbaf98f7aa2eb10cff3c29dbeca25d099e421a46a4cff40439aeaf3a25b77'; p.chmod(0o555)"
ENV PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1
EXPOSE 8088
ENTRYPOINT ["python3", "/app/entrypoint.py"]
