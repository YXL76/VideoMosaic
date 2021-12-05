// See [Search Images](https://pixabay.com/api/docs/#api_search_images)

import {
  copy,
  readerFromStreamReader,
} from "https://deno.land/std@0.117.0/streams/mod.ts";
import { extname, resolve } from "https://deno.land/std@0.117.0/path/mod.ts";

const API_KEY = Deno.env.get("API_KEY");
if (!API_KEY) Deno.exit(1);

const COLORS = [
  // "grayscale",
  // "transparent",
  "red",
  "orange",
  "yellow",
  "green",
  "turquoise",
  "blue",
  "lilac",
  "pink",
  "white",
  "gray",
  "black",
  "brown",
];
const PER_PAGE = 40;
const DIR_NAME = "pixabay";

const url = new URL("https://pixabay.com/api/");
url.searchParams.append("key", API_KEY);
url.searchParams.append("image_type", "photo");
url.searchParams.append("colors", "");
url.searchParams.append("per_page", PER_PAGE.toString());

console.log("Started");
try {
  // Deno.removeSync(DIR_NAME, { recursive: true });
  Deno.mkdirSync(DIR_NAME);
} catch (e) {
  console.log(e);
}

for (const color of COLORS) {
  console.log(`Searching for ${color}`);
  url.searchParams.set("colors", color);

  const res = await fetch(url, { keepalive: true });
  const { hits } = (await res.json()) as {
    hits: { id: number; webformatURL: string }[];
  };

  for (const { id, webformatURL } of hits) {
    console.log(`Downloading ${id}`);
    const url = webformatURL.replace("_1280.", "_300.");
    const path = resolve(DIR_NAME, `${id}${extname(webformatURL)}`);
    try {
      const res = await fetch(url, { keepalive: true });
      const file = Deno.createSync(path);
      const reader = readerFromStreamReader(res.body!.getReader());
      await copy(reader, file);
      file.close();

      console.log(`Saved ${id}`);
    } catch (err) {
      console.log(`Error ${id}: ${err}`);
      try {
        Deno.removeSync(path);
      } catch {
        //
      }
    }

    Deno.sleepSync(50);
  }

  console.log(`Saved ${color}`);
}

console.log("Finished");
