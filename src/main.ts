import "@fontsource/barlow/400.css";
import "@fontsource/barlow/500.css";
import "@fontsource/barlow/600.css";
import "@fontsource/barlow/700.css";
import "@fontsource/barlow-condensed/500.css";
import "@fontsource/barlow-condensed/600.css";
import "@fontsource/barlow-condensed/700.css";
import "./app.css";
import "flag-icons/css/flag-icons.min.css";
import App from "./App.svelte";

const app = new App({ target: document.getElementById("app")! });
export default app;
