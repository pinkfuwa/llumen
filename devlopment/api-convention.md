Backend API can be found at `./backend/src/routes`.

The convention is to use POST for JSON, GET for file download.

> IIRC there is a issue explaining why browser cannot POST to download file(due to how file chunking work on `HTTP/2` framing).

For path, we follow pattern `/api/{resource}/{ops}`.
