# NautilusTrader backtest/validation microservice — backtest only, no live orders.
FROM python:3.12-slim

WORKDIR /app

# nautilus_trader ships prebuilt manylinux wheels (Rust core); pandas/numpy pulled as deps.
RUN pip install --no-cache-dir \
        nautilus_trader==1.229.0 \
        fastapi \
        "uvicorn[standard]" \
        pandas

COPY backtest.py app.py ./

EXPOSE 8000
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
