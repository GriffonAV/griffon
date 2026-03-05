import React from "react";

interface PageProps {
  children?: React.ReactNode;
}

export const PluginsProvider: React.FC<PageProps> = ({ children }) => {
  return children;
};
