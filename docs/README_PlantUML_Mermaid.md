# Using PlantUML and Mermaid

I use some mermaid diagrams and many PlantUml diagrams _(when I find the mermaid diagrams lacking)_. See [mermaid](https://mermaid.js.org/intro/) for examples. I personally like and use their [Gantt](https://mermaid.js.org/syntax/gantt.html), [timelines](https://mermaid.js.org/syntax/timeline.html) and [block](https://mermaid.js.org/syntax/block.html) diagrams.

 - VSCode has mermaid support via extensions
 - Mermaid diagrams included using the ` ```mermaid` code-block show up in the Markdown preview
 - Github support: Github's markdown renderer, renders mermaid natively. This means that you can leave mermaid diagrams embedded in the markdown and web rendering will display the images. Awesome actually.

## PlantUML

I like plantuml for their `component diagrams`, `sequence diagrams`, `activity diagrams` and especially their `mind maps`. The points to note here is

 - VSCode has plantuml preview support _(I use the [extension by Jebbs](https://marketplace.visualstudio.com/items?itemName=jebbs.plantuml))_
 - Ideally you setup this extension by asking it to 
   - Use a Server
   - Use the default `http://www.plantuml.com/plantuml`
 - However, each character edit, triggers a POST to the server and of late _(Fall 2024)_, the server has finally started blocking requests that come in too quickly _(i.e., as you keep editig a doc and each character edit sends a request)_. You'll know this is happening when you get a _Sorry the server is down response_. To work around this, use a local server. Assuming you have a Linux or WSL distro
   - `docker pull plantuml/plantuml-server:jetty`
   - `docker run -d --name plantuml -p 8181:8080 plantuml/plantuml-server:jetty` for initial container creation
   - `docker start plantuml` for subsequent starts of the previously created container
   - 👉 If using WSL, there is some magic that goes on. When you launch the plantuml server listening on 0.0.0.0:8181, this shows up at the **WSL host/Windows** level as localhost:8181 as well as localhost:8181 at the WSL distro level. This is because windows performs automatic port-mapping of WSL services to the host: and this is why, VSCode running on windows can access the plantuml server on WSL via `localhost:8181`.
   - In VSCode, use `http://localhost:8181/` as the PlantUML server URL _(Sometimes, when this fails, 127.0.0.1:8181 works. Haven't looked into why)_.


### PlantUML in development

While authoring docs with plantuml diagrams. The most efficient option is to include the ` ```plantuml` code-blocks inline and view them inline via the preview _(Open preview to the side - <kbd>Ctrl+K</kbd>,<kbd>V</kbd> or top-right button-menu)_.

If you want your audience to view the images straight on github, you workaround github's lack of native plantuml rendering support by

 - Save the individiual plantuml blocks to `.puml` files: see [docs/base-rust-web-app/puml/](./base-rust-web-app/puml/).
 - Use a link to an image served by www.plantuml.com
   - Open the `.puml` file
   - use the `Context-menu / Generate URL for Current Diagram`
   - choose image format `png|svg|etc`
   - Copy the generated makrdown image link _(which contains the entire plantuml image as an encoded string)_ into your markdown text.   
 - Export the image and use that exported image
   - Open the `.puml` file
   - use the `Context-menu / Export Current Diagram`
   - choose export image format `png|svg|etc`
   - move the generated `out/../*.png` into your image folder _(img say)_ 
   - and finally link to it using `![Alt Text](./img/exportedImage.png)` in your markdown