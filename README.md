# header-injection Rust based WASM Filter

This filter extracts the body from a subsequent external service call and injects the body content as headers in the original request.

Refer to the flow below to get an understanding of this filter.

Built for Rust edition 2021 and Proxy-Wasm ABI v0.2.1.

Note here that the filter does expect an upstream called `postman-echo-service` for it to work. Refer to the flow diagram below.

## Build Instructions

### Update deps

```
bazel run //bazel/cargo:crates_vendor -- --repin
```

### Building Locally

```
bazel run //:header_injection
```

## Building And Pushing Remotely

Registry is assumed to be [webassemblyhub.io](webassemblyhub.io).

1. Sign up and register with [webassemblyhub.io](webassemblyhub.io/login/signup/)

2. Setup wasme plugin

    ```
    ./install_wasme_cli.sh
    ```

    add to `PATH`,
    ```
    export PATH=$HOME/.wasme/bin:$PATH
    ```

3. Login via `wasme` using the credentials used earlier to register to [webassemblyhub.io](webassemblyhub.io)

    ```
    wasme login -u <USERNAME> -p <PASSWORD>
    ```

4. Building the filter locally (Note: This does require Docker engine locally)

    ```
    wasme build rust -i gcr.io/solo-test-236622/wasm-builder:dev -t webassemblyhub.io/<org>/header-injection-plugin:0.1 -g header_injection -f header_injection.wasm
    ```

5. Pushing the image to [webassemblyhub.io](webassemblyhub.io)

    ```
    wasme push webassemblyhub.io/<org>/header-injection-plugin:0.1
    ```

## Testing

### Using a standard Envoy

Use the following Envoy configuration to test this filter,

```
admin:
  access_log_path: /dev/null
  address:
    socket_address:
      address: 0.0.0.0
      port_value: 19000
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address: { address: 0.0.0.0, port_value: 8080 }
      filter_chains:
        - filters:
          - name: envoy.filters.network.http_connection_manager
            typed_config:
              "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
              stat_prefix: ingress_http
              codec_type: AUTO
              route_config:
                name: test
                virtual_hosts:
                  - name: postman-echo
                    domains: ["*"]
                    routes:
                      - match: { prefix: "/" }
                        route:
                          cluster: postman-echo-service
                          auto_host_rewrite: true
              http_filters:
                - name: envoy.filters.http.wasm
                  typed_config:
                    "@type": type.googleapis.com/envoy.extensions.filters.http.wasm.v3.Wasm
                    config:
                        name: "header_injection"
                        root_id: "header_injection"
                        configuration:
                          "@type": "type.googleapis.com/google.protobuf.StringValue"
                          value: |
                            {}
                        vm_config:
                          vm_id: "header_injection"
                          code:
                            local:
                              # WASM plugin needs to be mounted via volumes
                              filename: "tmp/header_injection.wasm"
                - name: envoy.filters.http.router
                  typed_config:
                    "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
  clusters:
    - name: postman-echo-service
      connect_timeout: 5s
      type: LOGICAL_DNS
      lb_policy: ROUND_ROBIN
      dns_lookup_family: V4_ONLY
      load_assignment:
        cluster_name: postman-echo-service
        endpoints:
          - lb_endpoints:
            - endpoint:
                address:
                  socket_address:
                    address: postman-echo.com
                    port_value: 443
                    ipv4_compat: true
      transport_socket:
        name: envoy.transport_sockets.tls
        typed_config:
          "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext
          sni: postman-echo.com
```