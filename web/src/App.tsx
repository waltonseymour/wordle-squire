import React, { useEffect, useRef, useState } from "react";
import "./App.css";

type GuessState = "Correct" | "WrongPlace" | "Missing";

const Word: React.FC<{ guess?: string }> = (props) => {
  return (
    <div
      style={{
        display: "flex",
        gap: "5px",
      }}
    >
      <Tile result="Correct" letter={props.guess?.[0]} />
      <Tile letter={props.guess?.[1]} />
      <Tile letter={props.guess?.[2]} />
      <Tile letter={props.guess?.[3]} />
      <Tile letter={props.guess?.[4]} />
    </div>
  );
};

function getColor(state: GuessState) {
  switch (state) {
    case "Missing":
      return "#787c7e";
    case "WrongPlace":
      return "#c9b458";
    case "Correct":
      return "#6aaa64";
  }
}

const Tile: React.FC<{ letter?: string; result?: GuessState }> = (props) => {
  const result = props.result || "Missing";
  return (
    <div
      style={{
        display: "inline-flex",
        justifyContent: "center",
        alignItems: "center",
        width: "62px",
        height: "62px",
        background: getColor(result),
        fontSize: "2rem",
        lineHeight: "2rem",
        color: "white",
        textTransform: "uppercase",
        fontWeight: "bold",
      }}
    >
      {props.letter}
    </div>
  );
};

const App: React.FC = () => {
  const [guess, setGuess] = useState("");
  const guessRef = useRef("");

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "Backspace") {
        guessRef.current = guessRef.current.slice(0, -1);
        setGuess(guessRef.current);
      } else if (guessRef.current.length < 5 && e.key.length === 1) {
        guessRef.current += e.key;
        setGuess(guessRef.current);
      }
    };

    document.addEventListener("keydown", handler);

    return () => {
      document.removeEventListener("keypress", handler);
    };
  }, []);

  console.log(guessRef.current);

  return (
    <div style={{ display: "flex", flexDirection: "row" }}>
      <div
        className="guesses"
        style={{
          display: "flex",
          width: "100%",
          maxWidth: "500px",
          margin: "0 auto",
          justifyContent: "center",
        }}
      >
        <Word guess={guess} />
      </div>
      <div className="possible-solutions"></div>
    </div>
  );
};

export default App;
