package Services {
    part def Documented {
        doc /* Real documentation text here. */
    }
    part def Undocumented;
}
package System {
    part def Top {
        doc /* Top's own doc. */
        part x : Services::Documented;
        part y : Services::Undocumented;
    }
}
