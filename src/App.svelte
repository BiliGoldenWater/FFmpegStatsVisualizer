<script lang="ts">
  import {listen} from "@tauri-apps/api/event";

  import {onDestroy, onMount} from "svelte";
  import type {TLastStats, TParsedStats, TStats} from "./TApp";
  import * as echarts from "echarts";
  import {GridComponent, LegendComponent, TooltipComponent,} from "echarts/components";
  // @ts-ignore
  echarts.use([GridComponent, LegendComponent, TooltipComponent]);

  let element: HTMLDivElement;
  let instance: echarts.ECharts;

  let unlisten_stats: () => void;
  let loopId: number;
  let end: boolean = true;
  let index = 0;

  let current_stats: TParsedStats = {} as TParsedStats;
  let chart_data: { key: number; stats: TParsedStats }[] = [];
  let statsStr = "-";

  let statsToString = () =>
    `fps: ${current_stats.fps.toFixed(2)} ` +
    `bitrate: ${current_stats.bitrate.toFixed(2)}kbps ` +
    `speed: ${current_stats.speed.toFixed(2)}x ` +
    `dup_frames: ${current_stats.dup_frames} ` +
    `drop_frames: ${current_stats.drop_frames}`;

  let option = {
    tooltip: {},
    legend: {},
    xAxis: {
      data: [],
    },
    yAxis: {},
    series: [],
  };

  function reset() {
    current_stats = {
      fps: -1,
      bitrate: -1,
      speed: -1,
      dup_frames: -1,
      drop_frames: -1,
    };
    chart_data = [];
    index = 0;
  }

  // region loop
  function startLoop() {
    loopId = window.setTimeout(loop.bind(this), 500);
  }

  function loop() {
    chart_data.push({
      key: index++,
      stats: { ...current_stats },
    });

    option.xAxis.data = chart_data.map((v) => v.key.toString());
    option.series = Object.keys(current_stats).map((it) => ({
      name: it,
      type: "line",
      data: chart_data.map((v) => v.stats[it]),
    }));

    instance.setOption(option);

    if (!end) startLoop();
  }

  function stopLoop() {
    window.clearTimeout(loopId);
  }
  // endregion

  // region init
  onMount(async () => {
    reset();

    // region last stats
    let last_stats: TLastStats = {} as TLastStats;
    let last_ts: number;

    function resetStats() {
      last_stats = {
        frame: -1,
        total_size: { value: -1, last_time_micros: -1 },
        out_time_ms: -1,
        dup_frames: -1,
        drop_frames: -1,
      };
      last_ts = -1;
    }
    resetStats();
    // endregion

    unlisten_stats = await listen("ffmpeg_stats", (event) => {
      // region parse data
      let data = event.payload as { data: string; end: boolean };
      end = end || data.end;

      let stats: TStats = {
        frame: 0,
        total_size: 0,
        out_time_ms: 0,
        dup_frames: 0,
        drop_frames: 0,
      };

      for (const stats_item of data.data.split("\n")) {
        let item = stats_item.split("=");
        stats[item[0]] = parseFloat(item[1]);
      }
      // endregion

      // region check last stats
      if (data.end) {
        resetStats();
      } else {
        for (const key in last_stats) {
          if (last_stats[key] == -1 || last_stats[key]["value"] == -1) {
            last_stats = {
              ...stats,
              total_size: {
                value: stats.total_size,
                last_time_micros: stats.out_time_ms,
              },
            };
            last_ts = Date.now();
            return;
          }
        }
      }
      // endregion

      // region calc
      let ts = Date.now();
      let durationMs = ts - last_ts;
      let duration = durationMs / 1000;
      let total_size_updated =
        last_stats.total_size.value !== stats.total_size &&
        last_stats.total_size.last_time_micros !== stats.out_time_ms;

      // fps
      current_stats.fps = (stats.frame - last_stats.frame) / duration;
      // bitrate
      if (total_size_updated) {
        let duration =
          (stats.out_time_ms - last_stats.total_size.last_time_micros) / 1e6;
        let sizeIncrease = stats.total_size - last_stats.total_size.value;
        let bytesPerSecond = sizeIncrease / duration;
        let bitsPerSecond = bytesPerSecond * 8;
        current_stats.bitrate = bitsPerSecond / 1000;
      }
      // speed
      current_stats.speed =
        (stats.out_time_ms - last_stats.out_time_ms) / 1e6 / duration;
      // dup_frames
      current_stats.dup_frames = stats.dup_frames;
      // drop_frames
      current_stats.drop_frames = stats.drop_frames;

      //   console.log(statsToString());
      statsStr = statsToString();

      // region finally
      current_stats = { ...current_stats };

      last_stats = {
        ...stats,
        total_size: last_stats.total_size,
      };
      if (total_size_updated) {
        last_stats.total_size = {
          value: stats.total_size,
          last_time_micros: stats.out_time_ms,
        };
      }

      last_ts = ts;
      // endregion
      // endregion

      // region check init
      let inited = true;
      for (const key in current_stats) {
        inited = inited && current_stats[key] != -1;
      }
      if (end && inited && !data.end) {
        end = false;
        reset();
        startLoop();
      }
      // endregion

      if (data.end) console.log("ended");
    });

    instance = echarts.init(element);
    instance.setOption(option);
  });
  // endregion

  onDestroy(() => {
    unlisten_stats();
    stopLoop();
    instance.dispose();
  });
</script>

<svelte:window
  on:resize={() => {
    instance?.resize();
  }}
/>
<div class="app">
  <!--suppress CheckEmptyScriptTag -->
  <div class="chart" bind:this={element} />
  <div class="stats">{statsStr}</div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    align-items: center;

    height: 100vh;
  }

  .chart {
    flex-grow: 1;
    width: 100vw;
  }
</style>
