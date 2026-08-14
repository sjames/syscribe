package Services {
    part def Documented {
        doc /* Real documentation text here. */
    }
    part def Orphan {
        doc /* Never referenced anywhere. */
    }
}
package System {
    part def Top {
        doc /* Top's own doc. */
        part x : Services::Documented;
    }
}
