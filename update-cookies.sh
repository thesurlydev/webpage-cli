#!/usr/bin/env bash

# This script will update the cookies used by playwright if you are using the fetch arg.
# It will delete the cookies folder and then run hack-browser-data to get the latest cookies.
# See: https://github.com/moonD4rk/HackBrowserData

# When webpage-cli is called with the '--add-cookies' flag, it will add the
# cookies to the browser context based on the hostname of the url being fetched.

# After running this script, you can run webpage-cli like this:
# webpage-cli info --add-cookies --fetch --output-dir data <URL>
# webpage-cli info --add-cookies --fetch --output-dir data https://www.linkedin.com/jobs/search/?currentJobId=3653478524&f_TPR=a1690299919-&geoId=103644278&keywords=staff%20engineer&location=United%20States&refresh=true

set -e

rm -rf cookies
hack-browser-data-linux-amd64 --dir cookies --format json
