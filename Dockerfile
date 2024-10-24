FROM rust:latest as builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk@0.21.1
WORKDIR /usr/src/satisfactory-accounting
COPY . .
RUN cd satisfactory-accounting-app && trunk build --release

FROM nginx:latest
COPY ./default.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /usr/src/satisfactory-accounting/satisfactory-accounting-app/dist/ /usr/share/nginx/html/
