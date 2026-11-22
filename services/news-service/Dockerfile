FROM python:3.12-slim AS runtime
WORKDIR /app
ENV PYTHONUNBUFFERED=1 PYTHONDONTWRITEBYTECODE=1

RUN apt-get update && apt-get install -y --no-install-recommends gcc \
 && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt \
 && apt-get purge -y gcc && apt-get autoremove -y

COPY app ./app
# NB: Debian base already has a system user named "news" — use a distinct name.
RUN useradd -r -u 10003 appuser && chown -R appuser /app
USER appuser

EXPOSE 8000
CMD ["python", "-m", "app.main"]
