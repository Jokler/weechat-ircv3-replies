use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use weechat::hooks::{ModifierData, ModifierHook};
use weechat::{Args, Plugin, Weechat, plugin};

#[macro_export]
macro_rules! wdbg {
    () => {
        Weechat::print(&format!("[{}:{}:{}]", file!(), line!(), column!()));
    };

    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                Weechat::print(&format!("[{}:{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &&tmp as &dyn std::fmt::Debug,
                ));
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::wdbg!($val)),+,)
    };
}

struct ReplyPlugin {
    cache: Arc<Mutex<HashMap<String, Message>>>,
    privmsg_in_hook: ModifierHook,
    cap_hook: ModifierHook,
}

#[derive(Debug)]
struct Message {
    pub nick: String,
    pub text: String,
}

macro_rules! runwrap {
    ( $e:expr, $r:expr ) => {
        match $e {
            Some(x) => x,
            None => return $r,
        }
    };
}

impl Plugin for ReplyPlugin {
    fn init(weechat: &Weechat, _args: Args) -> Result<Self, ()> {
        Weechat::print("IRCv3 Reply Tags plugin initialized!");

        let cache = Arc::new(Mutex::new(HashMap::<String, Message>::new()));

        let cap_hook = ModifierHook::new(
            "irc_cap_sync_req",
            move |_weechat: &Weechat,
                  _modifier: &str,
                  data: Option<ModifierData>,
                  requested_caps: Cow<'_, str>| {
                // Logic to append 'message-tags' and 'echo-message'
                // let mut caps = requested_caps.to_string();
                // if modifier_data.contains("message-tags") && !caps.contains("message-tags") {
                //     caps.push_str(" message-tags");
                // }
                // if modifier_data.contains("echo-message") && !caps.contains("echo-message") {
                //     caps.push_str(" echo-message");
                // }
                // caps
                Some(requested_caps.to_string())
            },
        )?;

        let context_color = Weechat::color("darkgray");
        let nick_color = Weechat::color("cyan");
        let reset = Weechat::color("reset");

        let cache_ref = Arc::clone(&cache);
        let privmsg_in_hook = ModifierHook::new(
            "irc_in2_privmsg",
            move |weechat: &Weechat,
                  _modifier: &str,
                  data: Option<ModifierData>,
                  string: Cow<str>| {
                let mut parts = string.split(' ');
                let tags = runwrap!(parts.next(), Some(string.to_string()));
                let user = runwrap!(parts.next(), Some(string.to_string()));
                let _cmd = runwrap!(parts.next(), Some(string.to_string()));
                let target = runwrap!(parts.next(), Some(string.to_string()));
                let msg = parts.collect::<Vec<&str>>().join(" ");

                let mut unknown_reply = false;

                if tags.starts_with('@') && user.starts_with(':') {
                    let mut split = user[1..].split('!');
                    let nick = runwrap!(split.next(), Some(string.to_string())).to_string();
                    let targetbuffer = if target.starts_with('#') {
                        target.to_owned()
                    } else {
                        nick.clone()
                    };

                    for tag in tags[1..].split(';') {
                        let mut split = tag.split('=');

                        let Some(key) = split.next() else {
                            continue;
                        };
                        let Some(value) = split.next() else {
                            continue;
                        };

                        match key {
                            "msgid" => {
                                let mut cache = cache_ref.lock().unwrap();
                                if let Some(stripped) = msg.strip_prefix(':') {
                                    cache.insert(
                                        value.to_owned(),
                                        Message {
                                            nick: nick.clone(),
                                            text: stripped.to_owned(),
                                        },
                                    );
                                }
                            }
                            "+reply" => {
                                let cache = cache_ref.lock().unwrap();
                                if let Some(parent_msg) = cache.get(value) {
                                    let context_line = format!(
                                        "\t{}{} {}{}{} | {}{}",
                                        context_color,
                                        "↱",
                                        nick_color,
                                        parent_msg.nick,
                                        context_color,
                                        parent_msg.text,
                                        reset
                                    );

                                    if let Some(ModifierData::String(ref server)) = data {
                                        let buffername = format!("{server}.{targetbuffer}");
                                        if let Some(buffer) =
                                            weechat.buffer_search("irc", &buffername)
                                        {
                                            buffer.print(&context_line);
                                        } else {
                                            wdbg!("not found");
                                        }
                                    }
                                } else {
                                    unknown_reply = true;
                                }
                            }
                            _ => (),
                        }
                    }
                }

                if unknown_reply {
                    Some(format!("{string} (reply)"))
                } else {
                    Some(string.to_string())
                }
            },
        )?;

        Ok(Self {
            cache,
            privmsg_in_hook,
            cap_hook,
        })
    }
}

impl Drop for ReplyPlugin {
    fn drop(&mut self) {
        Weechat::print("Exiting Reply Plugin");
    }
}

plugin!(ReplyPlugin,
    name: "IRCv3 Reply Tags",
    author: "Jokler",
    description:"Adds functionality to understand IRCv3 tags and display them inline",
    version: "0.1.0"
);
