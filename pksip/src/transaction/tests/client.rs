// 17.1.1 INVITE Client Transaction
//
//                                |INVITE from TU
//              Timer A fires     |INVITE sent
//              Reset A,          V                      Timer B fires
//              INVITE sent +-----------+                or Transport Err.
//                +---------|           |---------------+inform TU
//                |         |  Calling  |               |
//                +-------->|           |-------------->|
//                          +-----------+ 2xx           |
//                             |  |       2xx to TU     |
//                             |  |1xx                  |
//     300-699 +---------------+  |1xx to TU            |
//    ACK sent |                  |                     |
// resp. to TU |  1xx             V                     |
//             |  1xx to TU  -----------+               |
//             |  +---------|           |               |
//             |  |         |Proceeding |-------------->|
//             |  +-------->|           | 2xx           |
//             |            +-----------+ 2xx to TU     |
//             |       300-699    |                     |
//             |       ACK sent,  |                     |
//             |       resp. to TU|                     |
//             |                  |                     |      NOTE:
//             |  300-699         V                     |
//             |  ACK sent  +-----------+Transport Err. |  transitions
//             |  +---------|           |Inform TU      |  labeled with
//             |  |         | Completed |-------------->|  the event
//             |  +-------->|           |               |  over the action
//             |            +-----------+               |  to take
//             |              ^   |                     |
//             |              |   | Timer D fires       |
//             +--------------+   | -                   |
//                                |                     |
//                                V                     |
//                          +-----------+               |
//                          |           |               |
//                          | Terminated|<--------------+
//                          |           |
//                          +-----------+

// ===== transaction state tests =====

#[tokio::test]
async fn invite_transition_to_calling_after_request_sent() { 
    
}