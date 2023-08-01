function build_row(zone, spots) {
    let innerRow = `
<div class="col">
    <h1 class="display-3 p-4 fw-bold m-2" style="color:#27a7a5">${zone}</h1>
</div>
<div class="col">
    <h2 class="display-4 text-end p-4 m-2" style="color:#27a7a5">${spots}</h2>
</div>
`;

    let row = document.createElement("div");
    row.innerHTML = innerRow;
    row.setAttribute("class", "row h-10 align-items-center");
    return row;
}

function setFullscreen() {
    let el = document.documentElement;
    let rfs = el.requestFullScreen || el.webkitRequestFullScreen || el.mozRequestFullScreen;
    rfs.call(el);
}

function setup() {
    addEventListener("click", setFullscreen);

    const evtSource = new EventSource("./listen");
    evtSource.onmessage = (event) => {
        let data = JSON.parse(event.data);

        // Build new element
        let new_section_node = document.createElement("div");
        new_section_node.setAttribute("class", "container-fluid");
        new_section_node.setAttribute("id", "section");
        new_section_node.setAttribute("style", "background-color: white; border-radius: 2%/5%;")

        data.sort((a, b) => a.order - b.order)

        // Generate child elements
        for (i in data) {
            let item = data[i];
            let count = item.capacity - item.count;
            let spots = "FULL"
            if (count > 0) {
                spots = `${count} Spots Left`
            }
            let child = build_row(item.name, spots);
            new_section_node.appendChild(child);
        }

        // Update changes
        let old_section_node = document.getElementById("section");
        old_section_node.replaceWith(new_section_node);
    };
}

setup()