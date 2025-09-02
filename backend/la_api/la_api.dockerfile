FROM rust:1.87

COPY ./ ./

WORKDIR "/la_api"

RUN apt-get update && apt-get install -y apt-transport-https 
RUN apt-get install -y build-essential \
    aptitude \
    git
RUN git clone https://github.com/CGuthmann/heimdall-libs.git /usr/lib/heimdall-libs

RUN cargo update

