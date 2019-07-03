# rust shopping bot

Port of https://developers.facebook.com/docs/messenger-platform/getting-started/webhook-setup to Rust

- https://messenger.com/platform
- https://developers.facebook.com/docs/messenger-platform/getting-started/quick-start
- https://developers.facebook.com/docs/messenger-platform/webhook
- https://developers.facebook.com/docs/graph-api/webhooks/getting-started/
- https://www.facebook.com/Shopping-bot-2185983088181959
- https://dashboard.heroku.com/apps/rust-shopping-bot
- https://rust-shopping-bot.herokuapp.com/
- https://github.com/alexreg/ergo-bot is using https://github.com/nocotan/rmessenger/
- got nothing https://github.com/fbsamples/messenger-platform-samples/pull/134/files#diff-dbc84bb7be1974a3b4ab3e5df3963f34R142

## Webhook

For `Callback URL` I have usedd https://rust-shopping-bot.herokuapp.com/webhook

- https://developers.facebook.com/docs/messenger-platform/getting-started/app-setup

## Deploy

``` bash
git push heroku
```

## Install and use

Install Rust using https://rustup.rs/ or other method.

Run with: `cargo run` in this directory.

## Verify token

Put token into `verify_token` file. Should be a random string
