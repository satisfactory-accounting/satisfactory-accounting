use gloo::file::Blob;
use log::{error, info, warn};
use web_sys::HtmlInputElement;
use yew::{
    classes, function_component, html, use_callback, AttrValue, Callback, Classes, Event, Html,
    Properties, TargetCast,
};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Callback to activate when the button is clicked.
    #[prop_or_default]
    pub onclick: Callback<()>,

    /// Whether the button should be disabled.
    #[prop_or_default]
    pub disabled: bool,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the main button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

/// Simple button with helper for material icons.
#[function_component]
pub fn Button(
    Props {
        children,
        onclick,
        disabled,
        class,
        title,
    }: &Props,
) -> Html {
    let disabled = *disabled;
    let class = classes!("Button", class.clone());
    let onclick = use_callback(onclick.clone(), |_, onclick| onclick.emit(()));

    html! {
        <button {class} {onclick} {disabled} {title}>
            { children.clone() }
        </button>
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct LinkProps {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Callback to activate when the button is clicked.
    #[prop_or_default]
    pub href: Option<AttrValue>,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the main button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,

    /// Html target property for the link.
    #[prop_or_default]
    pub target: Option<AttrValue>,
}

/// Simple button with helper for material icons.
#[function_component]
pub fn LinkButton(
    LinkProps {
        children,
        href,
        class,
        title,
        target,
    }: &LinkProps,
) -> Html {
    let class = classes!("Button", class.clone());

    html! {
        <a {class} {href} {target} {title}>
            { children.clone() }
        </a>
    }
}

/// Name and contents of an uploaded file.
#[derive(Debug)]
pub struct UploadedFile {
    /// The name of the file.
    pub name: String,
    /// The read file contents.
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Properties)]
pub struct UploadProps {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,

    /// Handler that receives the uploaded bytes.
    #[prop_or_default]
    pub onupload: Callback<UploadedFile>,
}

/// A button that accepts a file upload.
#[function_component]
pub fn UploadButton(
    UploadProps {
        children,
        class,
        title,
        onupload,
    }: &UploadProps,
) -> Html {
    let class = classes!("Button", class.clone());

    let onchange = use_callback(onupload.clone(), |e: Event, onupload| {
        let input = match e.target_dyn_into::<HtmlInputElement>() {
            Some(input) => input,
            None => {
                error!(
                    "Cannot handle file upload: Event target does not appear to be an \
                    HTMLInputElement"
                );
                return;
            }
        };
        let files = match input.files() {
            Some(files) => files,
            None => {
                warn!("HTMLInputElement did not have a 'files'");
                return;
            }
        };
        if files.length() > 1 {
            warn!("Received more than one input file. Taking only the first file.");
        }
        let file = match files.item(0) {
            Some(file) => file,
            None => {
                info!("No input files, doing nothing.");
                return;
            }
        };
        let name = file.name();
        let onupload = onupload.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let blob = Blob::from(file);
            let data = match gloo::file::futures::read_as_bytes(&blob).await {
                Ok(data) => data,
                Err(e) => {
                    warn!("Unable to read file contents: {e}");
                    return;
                }
            };
            onupload.emit(UploadedFile { name, data });
        })
    });

    html! {
        <label {class} {title}>
            <input type="file" accept="application/json" {onchange} />
            {children.clone()}
        </label>
    }
}
