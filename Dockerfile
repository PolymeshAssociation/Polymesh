FROM scratch

USER 4001:4001

COPY target/release/polymesh /usr/local/bin/polymesh

ENTRYPOINT ["/usr/local/bin/polymesh"]

