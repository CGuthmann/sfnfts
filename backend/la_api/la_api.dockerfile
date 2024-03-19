FROM rust:latest

COPY ./ ./

WORKDIR "/la_api"

RUN export HEIMDALL_RS_LIBS=/usr/lib/gcc/x86_64-linux-gnu/12/
RUN apt-get update && apt-get install -y apt-transport-https 
RUN apt-get install -y build-essential \
    aptitude \
    git
RUN git clone https://github.com/CGuthmann/heimdall-libs.git /usr/lib/heimdall-libs

RUN cargo update

