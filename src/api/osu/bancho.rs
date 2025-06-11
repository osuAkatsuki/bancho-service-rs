use crate::api::RequestContext;
use crate::events;
use crate::models::bancho::{BanchoRequest, BanchoResponse};

/// Controller for the osu! bancho protocol
pub async fn controller(ctx: RequestContext, request: BanchoRequest) -> BanchoResponse {
    events::handle_request(&ctx, request).await
}

pub async fn index() -> &'static str {
    r#"
       _/_/    _/                    _/                          _/        _/
    _/    _/  _/  _/      _/_/_/  _/_/_/_/    _/_/_/  _/    _/  _/  _/
   _/_/_/_/  _/_/      _/    _/    _/      _/_/      _/    _/  _/_/      _/
  _/    _/  _/  _/    _/    _/    _/          _/_/  _/    _/  _/  _/    _/
 _/    _/  _/    _/    _/_/_/      _/_/  _/_/_/      _/_/_/  _/    _/  _/
Click circle.. circle no click?

                          .. o  .
                         o.o o . o
                        oo...
                    __[]__
  jackson--> _\:D/_/o_o_o_|__     u wot m8
             \""""""""""""""/
              \ . ..  .. . /
 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                 ~~~~~~~
                               ~~~  🏯  ~~~
                             ~~~~  🏮  ~~~~
                               ~~~  🇨🇳  ~~~
                                 ~~~~~~~

Serving one handed osu! gamers since the dawn of time© Akatsuki, 2025
"#
}
