import React, { useState } from "react";
import { createRoot } from "react-dom/client";
function ControlledForm() {
  const [text, setText] = useState("old"),
    [choice, setChoice] = useState("a"),
    [checked, setChecked] = useState(false),
    [saved, setSaved] = useState("");
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        setSaved(JSON.stringify({ text, choice, checked }));
      }}
    >
      <label>
        Controlled input
        <input
          id="react-input"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      <select
        id="react-select"
        value={choice}
        onChange={(e) => setChoice(e.target.value)}
      >
        <option value="a">Alpha</option>
        <option value="b">Beta</option>
      </select>
      <input
        id="react-checkbox"
        type="checkbox"
        checked={checked}
        onChange={(e) => setChecked(e.target.checked)}
      />
      <button id="react-submit">Submit</button>
      <output id="react-state">
        {JSON.stringify({ text, choice, checked })}
      </output>
      <output id="react-saved">{saved}</output>
    </form>
  );
}
createRoot(document.getElementById("react-fixture")!).render(
  <ControlledForm />,
);
