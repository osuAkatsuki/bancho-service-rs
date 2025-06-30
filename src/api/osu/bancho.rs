use crate::api::RequestContext;
use crate::events;
use crate::models::bancho::{BanchoRequest, BanchoResponse};
use axum::response::Html;

/// Controller for the osu! bancho protocol
pub async fn controller(ctx: RequestContext, request: BanchoRequest) -> BanchoResponse {
    events::handle_request(&ctx, request).await
}

pub async fn index() -> Html<&'static str> {
    const RESPONSE: &str = r#"
<html><head><title>Bancho Server</title><style type='text/css'>body{width:30%;background:#222;color:#fff;}</style></head><body><pre>
       _/_/    _/                    _/                          _/        _/
    _/    _/  _/  _/      _/_/_/  _/_/_/_/    _/_/_/  _/    _/  _/  _/
   _/_/_/_/  _/_/      _/    _/    _/      _/_/      _/    _/  _/_/      _/
  _/    _/  _/  _/    _/    _/    _/          _/_/  _/    _/  _/  _/    _/
 _/    _/  _/    _/    _/_/_/      _/_/  _/_/_/      _/_/_/  _/    _/  _/
<b>Click circle.. circle no click?</b>

<marquee style='white-space:pre;'>
                          .. o  .
                         o.o o . o
                        oo...
                    __[]__
  jackson--> _\:D/_/o_o_o_|__     u wot m8
             \""""""""""""""/
              \ . ..  .. . /</marquee>
 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                 ~~~~~~~
                               ~~~  🏯  ~~~
                             ~~~~  🏮  ~~~~
                               ~~~  🇨🇳  ~~~
                                 ~~~~~~~

Serving one handed osu! gamers since the dawn of time &copy; Akatsuki, 2025
</pre>
</body>
</html>
"#;
    Html(RESPONSE)
}
