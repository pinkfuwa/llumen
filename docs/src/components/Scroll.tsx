import React from "react";
import styles from "./Scroll.module.css";

interface ScrollProps {
  imgs: string[];
  prefix?: string;
}

export function Scroll({ imgs, prefix = "" }: ScrollProps) {
  return (
    <div className={styles.container}>
      {imgs.map((img, index) => (
        <img src={prefix + img} className={styles.img} />
      ))}
    </div>
  );
}
