#FROM debian:stable-slim
FROM gcr.io/distroless/base


COPY --chown=4001:4001 . /
USER 4001:4001

ENTRYPOINT ["/usr/local/bin/polymesh"]
CMD [ "-d" "/var/lib/polymesh" ]

