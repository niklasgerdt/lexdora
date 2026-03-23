use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Document, Element, HtmlInputElement};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OrgOut {
    id: String,
    name: String,
    legal_entity_id: Option<String>,
}

fn document() -> Document {
    window().unwrap().document().unwrap()
}

fn set_text(el: &Element, text: &str) {
    el.set_text_content(Some(text));
}

async fn fetch_orgs() -> Result<Vec<OrgOut>, JsValue> {
    let resp = wasm_fetch_json("/orgs", None).await?;
    let orgs: Vec<OrgOut> = resp.into_serde().map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(orgs)
}

async fn create_org(name: &str, lei: Option<&str>) -> Result<(), JsValue> {
    let mut body = serde_json::json!({ "name": name });
    if let Some(v) = lei { body["legal_entity_id"] = serde_json::Value::String(v.to_string()); }
    let _resp = wasm_fetch_json(
        "/orgs",
        Some(FetchOptions {
            method: "POST",
            body: Some(serde_json::to_string(&body).unwrap()),
            content_type: Some("application/json"),
        }),
    ).await?;
    Ok(())
}

struct FetchOptions<'a> {
    method: &'a str,
    body: Option<String>,
    content_type: Option<&'a str>,
}

async fn wasm_fetch_json(url: &str, opts: Option<FetchOptions<'_>>) -> Result<JsValue, JsValue> {
    let win = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let mut init = web_sys::RequestInit::new();
    if let Some(o) = &opts {
        init.method(o.method);
        init.mode(web_sys::RequestMode::Cors);
        if let Some(b) = &o.body { init.body(Some(&JsValue::from_str(b))); }
    }
    let request = web_sys::Request::new_with_str_and_init(url, &init)?;
    if let Some(o) = &opts {
        if let Some(ct) = o.content_type {
            request.headers().set("Content-Type", ct)?;
        }
    }
    let resp_value = wasm_bindgen_futures::JsFuture::from(win.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    if !resp.ok() { return Err(JsValue::from_str(&format!("HTTP error {}", resp.status()))); }
    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    Ok(json)
}

fn render_orgs(list_el: &Element, orgs: &[OrgOut]) {
    // Clear existing
    list_el.set_inner_html("");
    for o in orgs {
        let li = document().create_element("li").unwrap();
        let lei = o.legal_entity_id.clone().unwrap_or_default();
        set_text(&li, &format!("{} — {}", o.name, lei));
        list_el.append_child(&li).unwrap();
    }
}

fn hook_create_form() {
    let doc = document();
    let name_input: HtmlInputElement = doc.get_element_by_id("org-name").unwrap().dyn_into().unwrap();
    let lei_input: HtmlInputElement = doc.get_element_by_id("org-lei").unwrap().dyn_into().unwrap();
    let button = doc.get_element_by_id("create-org").unwrap();
    let list_el = doc.get_element_by_id("org-list").unwrap();

    let closure = Closure::<dyn FnMut()>::new(move || {
        let name = name_input.value();
        let lei_v = lei_input.value();
        let list_el = list_el.clone();
        spawn_local(async move {
            if name.trim().is_empty() { return; }
            let _ = create_org(&name, if lei_v.trim().is_empty() { None } else { Some(&lei_v) }).await;
            if let Ok(orgs) = fetch_orgs().await {
                render_orgs(&list_el, &orgs);
            }
        });
    });
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}

async fn init_load() {
    let doc = document();
    let list_el = doc.get_element_by_id("org-list").unwrap();
    match fetch_orgs().await {
        Ok(orgs) => render_orgs(&list_el, &orgs),
        Err(e) => {
            let err = doc.get_element_by_id("error").unwrap();
            set_text(&err, &format!("Failed to load orgs: {:?}", e));
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    // Setup handlers and kick off initial load
    hook_create_form();
    spawn_local(async move { init_load().await; });
}
