#!/bin/bash

# 2000 solve ABCDEFGH 8 8 e107aa44a63ac64338cc632f264b6034218710adfcd28f8782d1aaabeb9d7fe9

# Build the project first
cargo build

# Create named pipes for each node
PIPE_2000="/tmp/node_2000_pipe"
PIPE_2001="/tmp/node_2001_pipe"
PIPE_2002="/tmp/node_2002_pipe"
PIPE_2003="/tmp/node_2003_pipe"
PIPE_2004="/tmp/node_2004_pipe"
PIPE_2005="/tmp/node_2005_pipe"

# Remove existing pipes
rm -f $PIPE_2000 $PIPE_2001 $PIPE_2002 $PIPE_2003 $PIPE_2004 $PIPE_2005

# Create new named pipes
mkfifo $PIPE_2000
mkfifo $PIPE_2001
mkfifo $PIPE_2002
mkfifo $PIPE_2003
mkfifo $PIPE_2004
mkfifo $PIPE_2005

# Cleanup function to kill all processes by port
cleanup() {
    echo ""
    echo "Stopping all nodes..."
    
    # Kill all background jobs
    jobs -p | xargs kill 2>/dev/null
    
    # Kill processes by port numbers
    lsof -ti:2000 2>/dev/null | xargs kill -9 2>/dev/null
    lsof -ti:2001 2>/dev/null | xargs kill -9 2>/dev/null
    lsof -ti:2002 2>/dev/null | xargs kill -9 2>/dev/null
    lsof -ti:2003 2>/dev/null | xargs kill -9 2>/dev/null
    lsof -ti:2004 2>/dev/null | xargs kill -9 2>/dev/null
    lsof -ti:2005 2>/dev/null | xargs kill -9 2>/dev/null
    
    # Kill any remaining node processes
    pkill -f "target/debug/node" 2>/dev/null
    
    # Remove named pipes
    rm -f $PIPE_2000 $PIPE_2001 $PIPE_2002 $PIPE_2003 $PIPE_2004 $PIPE_2005
    
    echo "All nodes stopped and ports freed"
    exit 0
}

# Set trap before starting processes
trap cleanup INT TERM EXIT

# Run multiple node instances in the background with named pipes for input
# Each output is prefixed with the port number for clarity

# Run first node
tail -f $PIPE_2000 | cargo run -- -p 2000 -f 2001,2005 2>&1 | sed 's/^/[Node 2000] /' &

# Wait a bit for first node to start
sleep 0.5

# Run second node
tail -f $PIPE_2001 | cargo run -- -p 2001 -f 2000,2002,2005 2>&1 | sed 's/^/[Node 2001] /' &

# Wait a bit
sleep 0.5

# Run third node (if needed)
tail -f $PIPE_2002 | cargo run -- -p 2002 -f 2001,2003,2005 2>&1 | sed 's/^/[Node 2002] /' &

# Wait a bit
sleep 0.5

# Run fourth node
tail -f $PIPE_2003 | cargo run -- -p 2003 -f 2002,2004,2005 --faker 2>&1 | sed 's/^/[Node 2003] /' &

# Wait a bit
sleep 0.5

# Run fifth node
tail -f $PIPE_2004 | cargo run -- -p 2004 -f 2003,2005 --faker 2>&1 | sed 's/^/[Node 2004] /' &

# Wait a bit
sleep 0.5

# Run sixth node
tail -f $PIPE_2005 | cargo run -- -p 2005 -f 2000,2001,2002,2003,2004 2>&1 | sed 's/^/[Node 2005] /' &

sleep 0.5

echo ""
echo "All nodes started"
echo "Commands:"
echo "  2000 <command> - Send command to node 2000"
echo "  2001 <command> - Send command to node 2001"
echo "  2002 <command> - Send command to node 2002"
echo "  2003 <command> - Send command to node 2003"
echo "  2004 <command> - Send command to node 2004"
echo "  2005 <command> - Send command to node 2005"
echo "  all <command>  - Send command to all nodes"
echo "  quit           - Stop all nodes and exit"
echo ""

# Read commands from user and send to appropriate node(s)
while read -r line; do
    if [ -z "$line" ]; then
        continue
    fi
    
    # Split into port and command
    port=$(echo "$line" | awk '{print $1}')
    command=$(echo "$line" | cut -d' ' -f2-)
    
    case "$port" in
        2000)
            echo "$command" > $PIPE_2000
            ;;
        2001)
            echo "$command" > $PIPE_2001
            ;;
        2002)
            echo "$command" > $PIPE_2002
            ;;
        2003)
            echo "$command" > $PIPE_2003
            ;;
        2004)
            echo "$command" > $PIPE_2004
            ;;
        2005)
            echo "$command" > $PIPE_2005
            ;;
        all)
            echo "$command" > $PIPE_2000
            echo "$command" > $PIPE_2001
            echo "$command" > $PIPE_2002
            echo "$command" > $PIPE_2003
            echo "$command" > $PIPE_2004
            echo "$command" > $PIPE_2005
            ;;
        quit|exit)
            cleanup
            ;;
        *)
            echo "Unknown port: $port. Use 2000, 2001, 2002, 2003, 2004, 2005, all, or quit"
            ;;
    esac
done
