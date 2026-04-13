"use client";

import { useEffect } from "react";

export default function Home() {
  useEffect(() => {
    window.location.replace("/dashboard");
  }, []);

  return (
    <div className="flex-center full-viewport">
      <div className="splash-loader" />
    </div>
  );
}
