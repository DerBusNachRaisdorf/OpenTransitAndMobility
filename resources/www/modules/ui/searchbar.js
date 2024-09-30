// inspired by: https://codepen.io/mey_mnry/pen/QWqPvox

export class SearchBarItem {
  name; // name to display
  data; // user data provided on select

  constructor(name, data = null) {
    this.name = name;
    this.data = data;
  }
}

export class SearchBar {
  #searchInput; // search input, containing the input element and the result box.
  #input; // text input of the search box.
  #resultBox; // displays all matching results.
  #onSelectFunction = async (_name, _userData) => {
    return;
  };
  #itemProviderFunction = async (_searchPattern) => [];
  #searchResultsTimestamp = Date.now();

  /**
   * Creates a new Search Bar.
   * @constructor
   * @param {Element} searchContainer - A container element for the search.
   */
  constructor(searchContainer) {
    // create dom elements
    this.#searchInput = document.createElement("div");
    this.#searchInput.className = "search-input";
    this.#input = document.createElement("input");
    this.#input.type = "text";
    this.#resultBox = document.createElement("div");
    this.#resultBox.className = "search-result-box";
    this.#searchInput.appendChild(this.#input);
    this.#searchInput.appendChild(this.#resultBox);
    searchContainer.appendChild(this.#searchInput);

    // on input
    this.#input.addEventListener("input", async (e) => {
      let searchPattern = e.target.value;
      let timestamp = Date.now();
      let results = await this.#itemProviderFunction(searchPattern);

      // only update if newer than current state.
      if (this.#searchResultsTimestamp > timestamp) {
        return;
      }

      this.#searchResultsTimestamp = timestamp;

      if (searchPattern) {
        // show suggestions
        let suggestionHtmlElements = results.map((data) => {
          return (data = `<li>` + data.name + "</li>");
        });
        if (!suggestionHtmlElements.length) {
          this.#searchInput.classList.remove("active");
        } else {
          this.#searchInput.classList.add("active");
        }
        this.#resultBox.innerHTML = suggestionHtmlElements.join("");

        // add on click events
        let suggestions = this.#resultBox.querySelectorAll("li");
        for (let i = 0; i < suggestions.length; i++) {
          let name = results[i].name;
          let data = results[i].data;
          suggestions[i].addEventListener("click", (e) => {
            this.#onSelectFunction(name, data);
            this.#input.value = "";
            this.#closeSearchBarResults();
          });
        }
      } else {
        this.#searchInput.classList.remove("active");
      }
    });

    let closeResultsFunc = () => this.#closeSearchBarResults();

    document.addEventListener("click", function (e) {
      closeResultsFunc();
    });
  }

  onItemSelected(func) {
    this.#onSelectFunction = func;
  }

  itemProvider(func) {
    this.#itemProviderFunction = func;
  }

  get placeholder() {
    return this.#input.placeholder;
  }

  set placeholder(value) {
    this.#input.placeholder = value;
  }

  #closeSearchBarResults() {
    this.#searchInput.classList.remove("active");
  }
}
