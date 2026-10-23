### Performance
#### The landing and the blog load 0.46 MB instead of 6.4 MB (2026-10-23)

Every public page downloaded and ran the whole application (3 MB of JavaScript) and the engine
(3 MB of WASM) before it settled: Lighthouse measured performance 52 on the landing and 35 on the
blog, with the blog's largest paint at 19 s on a mobile profile. The public pages now render on
their own with only their share of the dictionaries, and the application and the engine load on
the first app route; an address that is the application's mounts it at once, as before (#595).
