import React, { ReactNode } from "react";
import styles from "./Slide.module.css";

interface SlideProps {
  imgs: string[];
  prefix?: string;
}

export function Slide({ imgs, prefix = "" }: SlideProps) {
  return (
    <div className={styles.container}>
      {imgs.map((img, index) => (
        <img src={prefix + img} className={styles.img} />
      ))}
    </div>
  );
}
