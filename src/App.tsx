import React from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { motion, useMotionValue } from "motion/react";
import { cn } from "./lib/utils";
import { Button } from "./components/ui/button";

type SongData = {
  interval: Array<[number, number, string]>;
  error_message?: PlayerError;
};

enum PlayerError {
  NoPlayer = "NoPlayer",
  NoLyrics = "NoLyrics",
}

type Progress = number;

const initial: SongData = {
  interval: [[0, 0, ""]],
  error_message: undefined,
};

function App() {
  const [data, setData] = useState<SongData>(initial);
  const [progress, setProgress] = useState<Progress>(0);
  const scale = useMotionValue(progress);
  const dataChannel = useMemo(() => new Channel<SongData>(), []);
  const progressChannel = useMemo(() => new Channel<Progress>(), []);
  const lineRefs = useRef<(HTMLParagraphElement | null)[]>([]);

  useEffect(() => {
    const scrollToCurrentLine = () => {
      const currentLine = lineRefs.current[progress];
      if (currentLine) {
        currentLine.scrollIntoView({
          behavior: "smooth",
          block: "center",
        });
      }
    };

    window.addEventListener("resize", scrollToCurrentLine, true);

    scrollToCurrentLine();

    return () => {
      window.removeEventListener("resize", scrollToCurrentLine, true);
    };
  }, [progress]);

  useEffect(() => {
    dataChannel.onmessage = (m) => {
      setData(m);
    };

    progressChannel.onmessage = (prog) => {
      setProgress((prevProgress) => {
        if (prog !== prevProgress) {
          scale.set(prog);
          return prog;
        }
        return prevProgress;
      });
    };

    invoke("get", { c: dataChannel, p: progressChannel });
  }, [dataChannel, progressChannel, scale]);

  return (
    <main className="px-[10%] my-[50%] mb-8 text-2xl leading-[1.5] text-center seleciton-none text-[#FAFEFD] flex justify-center items-center flex-col h-full w-full text-stroke-3 pointer-events-none">
      <p>
        {data.error_message === PlayerError.NoPlayer
          ? "No active players."
          : ""}
      </p>
      <div>
        {data.error_message === PlayerError.NoLyrics ? (
          <>
            <p>Couldn&apos;t find lyrics for this song.</p>
            <Button
              className="pointer-events-auto"
              variant="default"
              onClick={() => {
                window.location.reload();
              }}
            >
              Press here to retry
            </Button>
          </>
        ) : (
          ""
        )}
      </div>
      {data.interval.map((line, i) => {
        return (
          <motion.p
            key={line[0]}
            className={cn(i !== progress ? "blur-[1.5px]" : "")}
            ref={(el) => (lineRefs.current[i] = el)}
            animate={
              i === progress
                ? { scale: 1.2, opacity: 1 }
                : { scale: 1, opacity: 0.6 }
            }
          >
            {line[2]}
          </motion.p>
        );
      })}
    </main>
  );
}

export default App;
