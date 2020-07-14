FROM gcr.io/distroless/static

COPY --chown 4001:4001 polymesh /usr/local/bin/polymesh
USER 4001:4001

ENTRYPOINT ["/usr/local/bin/polymesh"]

