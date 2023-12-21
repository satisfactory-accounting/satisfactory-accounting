FROM rust:latest as builder
RUN rustup target add wasm32-unknown-unknown
# Trunk 0.18.0 breaks static JS with bad minification.
RUN cargo install --locked trunk@0.17.5
WORKDIR /usr/src/satisfactory-accounting
COPY . .
RUN cd satisfactory-accounting-app && trunk build --release

FROM nginx:latest
COPY ./default.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /usr/src/satisfactory-accounting/satisfactory-accounting-app/dist/ /usr/share/nginx/html/
