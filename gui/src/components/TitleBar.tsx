import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IconMinimize, IconMaximize, IconRestore, IconClose } from "../icons";

export default function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    appWindow.isMaximized().then(setMaximized);
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then(setMaximized);
    });
    return () => { unlisten.then(fn => fn()); };
  }, [appWindow]);

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-drag" data-tauri-drag-region>
        <span className="titlebar-title">nz-switch</span>
      </div>
      <div className="titlebar-controls">
        <button className="titlebar-btn" onClick={() => appWindow.minimize()} title="最小化">
          <IconMinimize />
        </button>
        <button className="titlebar-btn" onClick={() => appWindow.toggleMaximize()} title={maximized ? "还原" : "最大化"}>
          {maximized ? <IconRestore /> : <IconMaximize />}
        </button>
        <button className="titlebar-btn titlebar-btn-close" onClick={() => appWindow.close()} title="关闭">
          <IconClose />
        </button>
      </div>
    </div>
  );
}
