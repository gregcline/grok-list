(ns gregcline.grok-list-ui.views
    (:require
     [re-frame.core :as re-frame]
     [gregcline.grok-list-ui.subs :as subs]
     ))

(defn main-panel []
  (let [name (re-frame/subscribe [::subs/name])]
    [:div
     [:h1 "Hello from " @name]
     ]))
