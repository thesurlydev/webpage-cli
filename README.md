# webpage-cli

Interrogate web pages for information. Based on the excellent [webpage](https://github.com/orottier/webpage-rs) library.

## Usage

```shell
Interrogate web pages for information

USAGE:
webpage-cli <SUBCOMMAND>

OPTIONS:
-h, --help       Print help information
-V, --version    Print version information

SUBCOMMANDS:
help    Print this message or the help of the given subcommand(s)
info    Prints information about a webpage
```

### Example output

Given the following command:
```shell
webpage-cli info https://en.wikipedia.org/wiki/Web_page
```

Results in:
```shell
{
  "http": {
    "ip": "198.35.26.96",
    "transfer_time": {
      "secs": 0,
      "nanos": 160569000
    },
    "redirect_count": 0,
    "content_type": "text/html; charset=UTF-8",
    "response_code": 200,
    "headers": [
      "HTTP/1.1 200 OK",
      "date: Mon, 27 Jun 2022 17:28:08 GMT",
      "vary: Accept-Encoding,Cookie,Authorization",
      "server: ATS/8.0.8",
      "x-content-type-options: nosniff",
      "content-language: en",
      "last-modified: Tue, 21 Jun 2022 19:34:03 GMT",
      "content-type: text/html; charset=UTF-8",
      "age: 21763",
      "x-cache: cp4030 miss, cp4028 hit/4",
      "x-cache-status: hit-front",
      "server-timing: cache;desc=\"hit-front\", host;desc=\"cp4028\"",
      "strict-transport-security: max-age=106384710; includeSubDomains; preload",
      "report-to: { \"group\": \"wm_nel\", \"max_age\": 86400, \"endpoints\": [{ \"url\": \"https://intake-logging.wikimedia.org/v1/events?stream=w3c.reportingapi.network_error&schema_uri=/w3c/reportingapi/network_error/1.0.0\" }] }",
      "nel: { \"report_to\": \"wm_nel\", \"max_age\": 86400, \"failure_fraction\": 0.05, \"success_fraction\": 0.0}",
      "set-cookie: WMF-Last-Access=27-Jun-2022;Path=/;HttpOnly;secure;Expires=Fri, 29 Jul 2022 12:00:00 GMT",
      "set-cookie: WMF-Last-Access-Global=27-Jun-2022;Path=/;Domain=.wikipedia.org;HttpOnly;secure;Expires=Fri, 29 Jul 2022 12:00:00 GMT",
      "accept-ch: Sec-CH-UA-Arch,Sec-CH-UA-Bitness,Sec-CH-UA-Full-Version-List,Sec-CH-UA-Model,Sec-CH-UA-Platform-Version",
      "permissions-policy: interest-cohort=(),ch-ua-arch=(self \"intake-analytics.wikimedia.org\"),ch-ua-bitness=(self \"intake-analytics.wikimedia.org\"),ch-ua-full-version-list=(self \"intake-analytics.wikimedia.org\"),ch-ua-model=(self \"intake-analytics.wikimedia.org\"),ch-ua-platform-version=(self \"intake-analytics.wikimedia.org\")",
      "x-client-ip: 1.1.1.1",
      "cache-control: private, s-maxage=0, max-age=0, must-revalidate",
      "set-cookie: GeoIP=US:WA:Seattle:46.01:-122.82:v4; Path=/; secure; Domain=.wikipedia.org",
      "accept-ranges: bytes",
      "content-length: 85465"
    ],
    "url": "https://en.wikipedia.org/wiki/Web_page",
    "body": "...omitted for brevity...",
  },
  "html": {
    "title": "Web page - Wikipedia",
    "description": null,
    "url": "https://en.wikipedia.org/wiki/Web_page",
    "feed": null,
    "language": "en",
    "text_content": "...omitted for brevity...",
    "meta": {
      "og:image:height": "466",
      "og:title": "Web page - Wikipedia",
      "og:type": "website",
      "ResourceLoaderDynamicStyles": "",
      "referrer": "origin-when-cross-origin",
      "charset": "UTF-8",
      "og:image": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Chromium_on_Ubuntu.png/640px-Chromium_on_Ubuntu.png",
      "format-detection": "telephone=no",
      "generator": "MediaWiki 1.39.0-wmf.16",
      "og:image:width": "640"
    },
    "opengraph": {
      "og_type": "website",
      "properties": {
        "title": "Web page - Wikipedia"
      },
      "images": [
        {
          "url": "https://upload.wikimedia.org/wikipedia/commons/0/01/Chromium_on_Ubuntu.png",
          "properties": {
            "height": "874",
            "width": "1200"
          }
        },
        {
          "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Chromium_on_Ubuntu.png/800px-Chromium_on_Ubuntu.png",
          "properties": {
            "height": "583",
            "width": "800"
          }
        },
        {
          "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Chromium_on_Ubuntu.png/640px-Chromium_on_Ubuntu.png",
          "properties": {
            "height": "466",
            "width": "640"
          }
        }
      ],
      "videos": [],
      "audios": []
    },
    "schema_org": [
      {
        "schema_type": "Article",
        "value": {
          "@context": "https://schema.org",
          "@type": "Article",
          "author": {
            "@type": "Organization",
            "name": "Contributors to Wikimedia projects"
          },
          "dateModified": "2022-06-21T19:33:43Z",
          "datePublished": "2001-12-30T14:30:33Z",
          "headline": "single document composed of HTML that is directly viewable via web browsers and accessible via the World Wide Web",
          "image": "https://upload.wikimedia.org/wikipedia/commons/0/01/Chromium_on_Ubuntu.png",
          "mainEntity": "http://www.wikidata.org/entity/Q36774",
          "name": "Web page",
          "publisher": {
            "@type": "Organization",
            "logo": {
              "@type": "ImageObject",
              "url": "https://www.wikimedia.org/static/images/wmf-hor-googpub.png"
            },
            "name": "Wikimedia Foundation, Inc."
          },
          "sameAs": "http://www.wikidata.org/entity/Q36774",
          "url": "https://en.wikipedia.org/wiki/Web_page"
        }
      },
      {
        "schema_type": "Article",
        "value": {
          "@context": "https://schema.org",
          "@type": "Article",
          "author": {
            "@type": "Organization",
            "name": "Contributors to Wikimedia projects"
          },
          "dateModified": "2022-06-21T19:33:43Z",
          "datePublished": "2001-12-30T14:30:33Z",
          "headline": "single document composed of HTML that is directly viewable via web browsers and accessible via the World Wide Web",
          "image": "https://upload.wikimedia.org/wikipedia/commons/0/01/Chromium_on_Ubuntu.png",
          "mainEntity": "http://www.wikidata.org/entity/Q36774",
          "name": "Web page",
          "publisher": {
            "@type": "Organization",
            "logo": {
              "@type": "ImageObject",
              "url": "https://www.wikimedia.org/static/images/wmf-hor-googpub.png"
            },
            "name": "Wikimedia Foundation, Inc."
          },
          "sameAs": "http://www.wikidata.org/entity/Q36774",
          "url": "https://en.wikipedia.org/wiki/Web_page"
        }
      }
    ]
  }
}
```
