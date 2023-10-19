FROM ubuntu:jammy
RUN apt update && apt install -y ca-certificates
ADD target/release/tm-grpc /usr/bin/tm-grpc
ADD target/release/trackdump /usr/bin/trackdump
RUN mkdir /etc/tangomike /var/lib/tangomike
ADD tm-grpc/tangomike.toml /etc/tangomike/tm-grpc.toml
CMD [ "/usr/bin/tm-grpc", "-c", "/etc/tangomike/tm-grpc.toml" ]
