## Project Model

- Zustand
  - Zeitgebundene Datenstruktur
  - Parameter
    - Werden durch externe Datenquellen beschrieben
  - Konstanten
    - werden zur Übersetzungszeit berechnet
    - sind global
  - Variablen
    - können vom Model gelesen und geschrieben werden

- Datentypen
  - klar definierte Menge einer Datenstruktur
  - Algebra zur Konstruktion neuer Typen

- Funktionen
  - spezielle Datenstrukturen welche ein Programm speichern
  - bilden Eingabeparameter auf Ausgabeparameter ab
  - bilden Datentypen auf Datentypen ab
  - rein funktional

- Automatische Tests
  - Tests werden durch Zustandsbedingungen formuliert
  - Tests definieren die Menge der Zielzustände

- Zustandsbedingungen
  - Zustandsbedingungen entscheiden für einen gegebenen Zustand ob er eine
    Bedingung erfüllt oder nicht
  - Zustandsbedingungen werden durch Funktionen über Variablen definiert


- Typdefinition
  - Ein Typ wird als eine Menge von Typkonstruktoren definiert
  - Ein Typkonstruktor erzeugt einen Wert dieses Typen
