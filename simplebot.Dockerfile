FROM python:3.10.1-slim-buster
WORKDIR /app
COPY simplebot.py simplebot.py
CMD python simplebot.py
