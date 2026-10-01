<script lang="ts">
  import { caBand } from "../format";
  /** Current ability shown as a pinned race bib — the app's signature mark. */
  export let value: number;
  export let size: "s" | "m" | "l" = "s";
  $: band = caBand(value);
  $: text = value ? (Math.round(value * 10) / 10).toString() : "–";
</script>

<span class="bib {size}" title="Current ability {value}">
  <span class="pin a"></span><span class="pin b"></span>
  <span class="n">{text}</span>
  <span class="band" style="background:{band}"></span>
</span>

<style>
  .bib {
    position: relative; display: inline-flex; align-items: center; justify-content: center;
    background: var(--blanc); color: #11151a; border-radius: 3px;
    font-family: var(--font-cond); font-weight: 700; line-height: 1;
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.35); overflow: hidden; flex-shrink: 0;
  }
  .s { min-width: 42px; height: 22px; font-size: 15px; padding: 0 6px 2px; }
  .m { min-width: 58px; height: 34px; font-size: 22px; padding: 0 9px 3px; }
  .l { min-width: 92px; height: 58px; font-size: 38px; padding: 0 13px 6px; border-radius: 5px; }
  .band { position: absolute; left: 0; right: 0; bottom: 0; height: 3px; }
  .l .band { height: 6px; }
  .s .pin { display: none; }
  .pin { position: absolute; top: 3px; width: 3px; height: 3px; border-radius: 50%; background: #b9c0c9; }
  .m .pin, .l .pin { width: 4px; height: 4px; top: 4px; }
  .pin.a { left: 3px; } .pin.b { right: 3px; }
  .l .pin.a { left: 7px; } .l .pin.b { right: 7px; }
</style>
