import { useEffect, useRef, useState } from "react";
import type { Failure } from "./errors";
import { isFullscreen, onFullscreenChange, setFullscreen as setWindowFullscreen } from "./platform";

/**
 * The window in fullscreen, which brings focus mode with it; leaving it puts
 * focus mode back the way it was. Escape in a browser leaves fullscreen
 * without asking, so the state follows the window.
 */
export function useExperimentFullscreen(focusMode: boolean, setFocusMode: (value: boolean) => void, onError: (failure: Failure) => void) {
  const [fullscreen, setFullscreen] = useState(false);
  const focusBeforeFullscreen = useRef(false);

  useEffect(() => {
    isFullscreen().then(setFullscreen).catch(() => {});
    return onFullscreenChange(() => { isFullscreen().then(setFullscreen).catch(() => {}); });
  }, []);

  const toggleFullscreen = async () => {
    try {
      await setWindowFullscreen(!fullscreen);
      if (fullscreen) setFocusMode(focusBeforeFullscreen.current);
      else { focusBeforeFullscreen.current = focusMode; setFocusMode(true); }
      setFullscreen(!fullscreen);
    }
    catch (error) { onError(error); }
  };

  /** Escape on the canvas: out of fullscreen, back to the focus mode from before. */
  const exitFullscreen = () => {
    setWindowFullscreen(false).then(() => { setFullscreen(false); setFocusMode(focusBeforeFullscreen.current); }).catch(() => {});
  };

  return { fullscreen, toggleFullscreen, exitFullscreen };
}
