import { homeDir } from "@tauri-apps/api/path";
import { SetStateAction } from "jotai";
import { StatusInfo, StatusInfoContent, StatusInfoType } from "../types";

export async function trimHomePath(pathStr: string) {
  const home = await homeDir();
  return pathStr.replace(home, "~/");
}

export function triggerTempStatusPanel(
  setter: (update: SetStateAction<StatusInfo>) => void,
  type: StatusInfoType,
  content: StatusInfoContent
) {
  setter({ type, content });
  setTimeout(() => setter({ type: "normal", content: "done" }), 5000);
}
