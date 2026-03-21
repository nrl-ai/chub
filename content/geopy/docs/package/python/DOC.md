---
name: package
description: "geopy Python package guide for geocoding, reverse geocoding, distance calculations, adapters, and rate limiting"
metadata:
  languages: "python"
  versions: "2.4.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "geopy,python,geocoding,reverse-geocoding,location,distance,nominatim"
---

# geopy Python Package Guide

## Golden Rule

Use `geopy` as a client library for third-party geocoding services, not as a service or dataset by itself. Pick the specific geocoder class your project uses, pass its credentials and request options explicitly, set a real `user_agent` for `Nominatim`, and rate-limit any batch traffic to stay within the provider's terms and quotas.

## Install

Pin the package version your project expects:

```bash
python -m pip install "geopy==2.4.1"
```

Common alternatives:

```bash
uv add "geopy==2.4.1"
poetry add "geopy==2.4.1"
```

For the async adapter:

```bash
python -m pip install "geopy[aiohttp]==2.4.1"
```

## Setup And Initialization

`geopy` exposes many provider-specific geocoder classes through `geopy.geocoders`. Start with the concrete service you actually use.

### Nominatim

`Nominatim` is the most common example in code snippets, but it requires a custom `user_agent`. In geopy 2.x, using the default or a sample user agent raises `geopy.exc.ConfigurationError`.

```python
from geopy.geocoders import Nominatim

geocoder = Nominatim(
    user_agent="my-app/1.0 (contact@example.com)",
    timeout=10,
)
```

### API-key geocoder

Credentials are not global to `geopy`; they belong to the provider class.

```python
import os
from geopy.geocoders import GoogleV3

geocoder = GoogleV3(
    api_key=os.environ["GOOGLE_MAPS_API_KEY"],
    timeout=10,
)
```

## Core Usage

### Geocode an address

`geocode()` usually returns `None`, a single `Location`, or a list of `Location` objects when `exactly_one=False`.

```python
from geopy.geocoders import Nominatim

geocoder = Nominatim(user_agent="my-app/1.0")

location = geocoder.geocode(
    "1600 Amphitheatre Parkway, Mountain View, CA",
    exactly_one=True,
    addressdetails=True,
    language="en",
)

if location is None:
    raise LookupError("Address not found")

print(location.address)
print(location.latitude, location.longitude)
print(location.raw)
```

### Reverse geocode coordinates

```python
from geopy.geocoders import Nominatim

geocoder = Nominatim(user_agent="my-app/1.0")

location = geocoder.reverse(
    (37.4221, -122.0841),
    exactly_one=True,
    language="en",
)

if location:
    print(location.address)
```

### Request multiple matches

```python
from geopy.geocoders import Nominatim

geocoder = Nominatim(user_agent="my-app/1.0")

matches = geocoder.geocode("Springfield", exactly_one=False, limit=5) or []

for item in matches:
    print(item.address, item.latitude, item.longitude)
```

### Compute distances

`geopy.distance.distance(...)` currently uses the geodesic algorithm by default. Use `great_circle(...)` only when that approximation is acceptable.

```python
from geopy.distance import distance, geodesic, great_circle

newport_ri = (41.49008, -71.312796)
cleveland_oh = (41.499498, -81.695391)

print(distance(newport_ri, cleveland_oh).km)
print(geodesic(newport_ri, cleveland_oh).miles)
print(great_circle(newport_ri, cleveland_oh).miles)
```

## Configuration And Request Defaults

Use constructor arguments for per-instance defaults and method kwargs for per-call overrides.

Common constructor parameters across geocoders:

- `timeout`: seconds before `GeocoderTimedOut`; the global default is `1`
- `user_agent`: HTTP user agent string; mandatory for `Nominatim`
- `proxies`: proxy mapping
- `ssl_context`: custom SSL context
- `adapter_factory`: choose the HTTP adapter implementation
- `domain` and `scheme`: override provider endpoint details when the class supports them

You can set defaults once through `geopy.geocoders.options`:

```python
import geopy.geocoders
from geopy.geocoders import Nominatim

geopy.geocoders.options.default_user_agent = "my-app/1.0"
geopy.geocoders.options.default_timeout = 7

geocoder = Nominatim()
```

In 2.x, service-specific request parameters generally belong on `geocode(...)` or `reverse(...)`, not on the geocoder constructor.

## Async Usage

Async geocoding uses `AioHTTPAdapter` and `async with`. Install the `aiohttp` extra first.

```python
import asyncio

from geopy.adapters import AioHTTPAdapter
from geopy.geocoders import Nominatim

async def main() -> None:
    async with Nominatim(
        user_agent="my-app/1.0",
        adapter_factory=AioHTTPAdapter,
    ) as geocoder:
        location = await geocoder.geocode("Berlin")
        if location:
            print(location.address)

asyncio.run(main())
```

## Rate Limiting And Batch Work

Respect the provider's usage policy. `RateLimiter` and `AsyncRateLimiter` are the normal way to slow down repeated lookups and retry `GeocoderServiceError` failures.

### Synchronous rate limiting

```python
from geopy.extra.rate_limiter import RateLimiter
from geopy.geocoders import Nominatim

geocoder = Nominatim(user_agent="my-app/1.0")
geocode = RateLimiter(
    geocoder.geocode,
    min_delay_seconds=1,
    max_retries=2,
    error_wait_seconds=5,
)

for query in ["Paris", "Berlin", "Tokyo"]:
    location = geocode(query)
    print(location.address if location else None)
```

### Asynchronous rate limiting

```python
import asyncio

from geopy.adapters import AioHTTPAdapter
from geopy.extra.rate_limiter import AsyncRateLimiter
from geopy.geocoders import Nominatim

async def main() -> None:
    async with Nominatim(
        user_agent="my-app/1.0",
        adapter_factory=AioHTTPAdapter,
    ) as geocoder:
        geocode = AsyncRateLimiter(geocoder.geocode, min_delay_seconds=1)
        results = await asyncio.gather(*(geocode(q) for q in ["Paris", "Berlin", "Tokyo"]))
        for location in results:
            print(location.address if location else None)

asyncio.run(main())
```

## Exceptions And Failure Handling

Useful exceptions from `geopy.exc` to handle explicitly:

- `GeocoderTimedOut`: request exceeded the configured timeout
- `GeocoderUnavailable`: provider could not be reached
- `GeocoderServiceError`: service-side error class retried by the rate limiters
- `GeocoderQueryError`: bad input or invalid request shape
- `ConfigurationError`: invalid client configuration, including invalid `Nominatim` user agents

Minimal pattern:

```python
from geopy.exc import GeocoderServiceError, GeocoderTimedOut
from geopy.geocoders import Nominatim

geocoder = Nominatim(user_agent="my-app/1.0", timeout=10)

try:
    location = geocoder.geocode("Lisbon")
except GeocoderTimedOut:
    location = None
except GeocoderServiceError as exc:
    raise RuntimeError(f"geocoding failed: {exc}") from exc
```

## Common Pitfalls

- `geopy` does not give you geocoding data by itself. You must choose a real provider such as `Nominatim`, `GoogleV3`, `OpenMapQuest`, or `GeoNames`.
- Do not use `Nominatim()` without a real `user_agent`.
- Do not batch large jobs without rate limiting and provider-specific ToS review.
- `geocode()` can return `None`; guard before reading `.latitude`, `.longitude`, or `.raw`.
- In 2.x, most service-specific arguments must be passed as keyword arguments, not positional arguments.
- The default timeout is very short (`1` second). Raise it for real network conditions.
- `timeout=None` disables timeouts in 2.x; it does not restore the default timeout.
- Distance calculations ignore altitude and raise `ValueError` for points with different altitudes.
- Many old blog posts still use pre-2.0 imports or constructor signatures. Prefer the current class import from `geopy.geocoders`.

## Version-Sensitive Notes For geopy 2.4.1

- `2.4.1` is in the 2.x line documented at the stable docs root and changelog.
- Async support, adapters, `AsyncRateLimiter`, and the `geopy[aiohttp]` extra were introduced in `2.0`.
- `RequestsAdapter` is the default adapter when `requests` is installed; otherwise geopy falls back to `URLLibAdapter`.
- Since `2.0`, `Nominatim` rejects default or sample user agents with `ConfigurationError`.
- Since `2.0`, service-specific request parameters belong on `geocode(...)` and `reverse(...)`, not on geocoder constructors.
- Since `2.0`, `geopy.distance.vincenty` is gone; use `geopy.distance.geodesic`.
- Old module paths such as `geopy.geocoders.osm.Nominatim` and `geopy.geocoders.googlev3.GoogleV3` remain only for compatibility and are slated for removal in geopy `3`.
- `2.3.0` dropped Python `3.5` and `3.6`; `2.4.0` added Python `3.12` support.

## Current Links

- Official docs: `https://geopy.readthedocs.io/en/stable/`
- Changelog: `https://geopy.readthedocs.io/en/stable/changelog_2xx.html`
- Package registry: `https://pypi.org/project/geopy/`
