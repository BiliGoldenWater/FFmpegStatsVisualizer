export type TStats = {
  frame: number;
  total_size: number;
  out_time_ms: number;
  dup_frames: number;
  drop_frames: number;
};

export type TLastStats = {
  frame: number;
  total_size: { value: number; last_time_µs: number };
  out_time_ms: number;
  dup_frames: number;
  drop_frames: number;
};

export type TParsedStats = {
  fps: number;
  bitrate: number;
  speed: number;
  dup_frames: number;
  drop_frames: number;
};
