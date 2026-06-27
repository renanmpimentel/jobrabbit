// Navigation context: allows any page to switch the active tab.
import { createContext, useContext, type ReactNode } from "react";

const NavCtx = createContext<(id: string) => void>(() => {});

export function useNav(): (id: string) => void {
  return useContext(NavCtx);
}

export function NavProvider({
  navigate,
  children,
}: {
  navigate: (id: string) => void;
  children: ReactNode;
}) {
  return <NavCtx.Provider value={navigate}>{children}</NavCtx.Provider>;
}
