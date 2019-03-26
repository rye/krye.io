# krye.io &cdot; [![dependency status](https://deps.rs/repo/github/rye/krye.io/status.svg)](https://deps.rs/repo/github/rye/krye.io)

The managerial/front-end part of my personal website.
More than anything, a set of scripts to quickly and efficiently generate content pages from source Markdown, templates.

This is intended primarily to rest as a simple HTTP/2 server with content generated into a cache.
The primary content for this site gets served from a GitHub Pages-as-a-CDN repository, and this repository encapsulates that content.

A separate repository, `api.krye.io` is/was created for the purpose of dynamic content population and storage.
The API itself is a separate web application system which is requested-from by client-side JS.
Things like database queries are done this way and a JSON API is exposed. Endpoints can thus be published here without interrupting or coinciding with content changes.
